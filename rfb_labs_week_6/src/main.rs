use anyhow::{Context, Result, bail};
use bdk_bitcoind_rpc::Emitter;
use bdk_wallet::rusqlite::Connection;
use bdk_wallet::{
    KeychainKind, Wallet,
    bitcoin::{Address, Amount, FeeRate, Network, bip32::Xpriv},
    signer::SignOptions,
};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use clap::{Parser, Subcommand};
use std::{
    path::PathBuf,
    str::FromStr,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

const NETWORK: Network = Network::Regtest;
const SEND_AMOUNT: u64 = 50_000;
const RPC_URL: &str = "127.0.0.1:18443";

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long, default_value = RPC_URL, env = "BITCOIN_RPC_URL")]
    rpc_url: String,

    #[arg(long, env = "BITCOIN_RPC_COOKIE")]
    rpc_cookie: Option<PathBuf>,

    #[arg(long, env = "BITCOIN_RPC_USER")]
    rpc_user: Option<String>,

    #[arg(long, env = "BITCOIN_RPC_PASS")]
    rpc_pass: Option<String>,

    #[arg(long, default_value = "wallet.sqlite")]
    db: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Demo,
    Address {
        #[arg(long)]
        change: bool,
    },
    Sync,
    Send {
        address: String,
        amount_sat: u64,
    },
}

fn rpc_client(args: &Args) -> Result<Client> {
    let auth = match (&args.rpc_cookie, &args.rpc_user, &args.rpc_pass) {
        (Some(path), _, _) => Auth::CookieFile(path.clone()),
        (_, Some(user), Some(pass)) => Auth::UserPass(user.clone(), pass.clone()),
        (None, None, None) => Auth::None,
        _ => bail!("RPC authentication requires both BITCOIN_RPC_USER and BITCOIN_RPC_PASS"),
    };

    Ok(Client::new(&args.rpc_url, auth)?)
}

fn load_or_create_wallet(
    db: &mut Connection,
) -> Result<(bdk_wallet::PersistedWallet<Connection>, [u8; 32])> {
    db.execute(
        "CREATE TABLE IF NOT EXISTS wallet_metadata (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            seed_hex TEXT NOT NULL
        )",
        [],
    )?;

    let seed_hex: Option<String> = db
        .query_row(
            "SELECT seed_hex FROM wallet_metadata WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .optional()?;

    let seed = match seed_hex {
        Some(hex_seed) => {
            let bytes = hex::decode(hex_seed).context("stored wallet seed is invalid hex")?;
            bytes
                .try_into()
                .map_err(|_| anyhow::anyhow!("stored wallet seed must be 32 bytes"))?
        }
        None => {
            let mut seed = [0u8; 32];
            getrandom::fill(&mut seed)
                .map_err(|e| anyhow::anyhow!("failed to generate wallet seed: {e:?}"))?;
            db.execute(
                "INSERT INTO wallet_metadata (id, seed_hex) VALUES (1, ?1)",
                [&hex::encode(seed)],
            )?;
            seed
        }
    };

    let master = Xpriv::new_master(NETWORK, &seed)?;
    let external = format!("wpkh({}/84'/1'/0'/0/*)", master);
    let internal = format!("wpkh({}/84'/1'/0'/1/*)", master);

    let wallet = match Wallet::load()
        .descriptor(KeychainKind::External, Some(external.clone()))
        .descriptor(KeychainKind::Internal, Some(internal.clone()))
        .extract_keys()
        .check_network(NETWORK)
        .load_wallet(db)?
    {
        Some(wallet) => wallet,
        None => Wallet::create(external, internal)
            .network(NETWORK)
            .create_wallet(db)?,
    };

    Ok((wallet, seed))
}

fn sync_wallet(
    wallet: &mut bdk_wallet::PersistedWallet<Connection>,
    db: &mut Connection,
    rpc: Arc<Client>,
) -> Result<()> {
    let tip = wallet.latest_checkpoint().clone();

    let mut emitter = Emitter::new(
        rpc,
        tip.clone(),
        tip.height(),
        wallet
            .transactions()
            .filter(|tx| tx.chain_position.is_unconfirmed()),
    );

    while let Some(event) = emitter.next_block()? {
        wallet.apply_block_connected_to(
            &event.block,
            event.block_height(),
            event.connected_to(),
        )?;
        wallet.persist(db)?;
    }

    let mempool = emitter.mempool()?;
    wallet.apply_evicted_txs(mempool.evicted);
    wallet.apply_unconfirmed_txs(mempool.update);
    wallet.persist(db)?;

    Ok(())
}

fn receive_address(
    wallet: &mut bdk_wallet::PersistedWallet<Connection>,
    db: &mut Connection,
) -> Result<Address> {
    let address = wallet.reveal_next_address(KeychainKind::External).address;
    wallet.persist(db)?;
    Ok(address)
}

fn change_address(
    wallet: &mut bdk_wallet::PersistedWallet<Connection>,
    db: &mut Connection,
) -> Result<Address> {
    let address = wallet.reveal_next_address(KeychainKind::Internal).address;
    wallet.persist(db)?;
    Ok(address)
}

fn send(
    wallet: &mut bdk_wallet::PersistedWallet<Connection>,
    db: &mut Connection,
    rpc: &Client,
    destination: Address,
    amount_sat: u64,
) -> Result<bdk_wallet::bitcoin::Txid> {
    let mut builder = wallet.build_tx();
    builder
        .add_recipient(destination.script_pubkey(), Amount::from_sat(amount_sat))
        .fee_rate(FeeRate::from_sat_per_vb(1).context("invalid fee rate")?);

    let mut psbt = builder.finish()?;
    let finalized = wallet.sign(&mut psbt, SignOptions::default())?;

    if !finalized {
        bail!("wallet could not finalize the transaction");
    }

    let tx = psbt.extract_tx()?;
    let txid = tx.compute_txid();

    rpc.send_raw_transaction(&tx)?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before UNIX epoch")?
        .as_secs();

    wallet.apply_unconfirmed_txs([(tx.clone(), now)]);
    wallet.persist(db)?;

    Ok(txid)
}

fn demo(args: &Args) -> Result<()> {
    let rpc = Arc::new(rpc_client(args)?);

    let chain = rpc.get_blockchain_info()?;
    if chain.chain != bitcoincore_rpc::bitcoin::Network::Regtest {
        bail!("this assignment demo requires a regtest Bitcoin Core node");
    }

    println!("Connected to Bitcoin Core: regtest");

    let mut db = Connection::open(&args.db)?;
    let (mut wallet, _seed) = load_or_create_wallet(&mut db)?;

    let receive = receive_address(&mut wallet, &mut db)?;
    let change = change_address(&mut wallet, &mut db)?;

    println!("External/receiving address: {receive}");
    println!("Internal/change address:     {change}");

    sync_wallet(&mut wallet, &mut db, rpc.clone())?;

    if wallet.balance().trusted_spendable().to_sat() < SEND_AMOUNT + 10_000 {
        println!("Funding wallet with regtest mining...");
        rpc.generate_to_address(101, &receive)?;
        sync_wallet(&mut wallet, &mut db, rpc.clone())?;
    }

    let before = wallet.balance();
    let utxos_before = wallet.list_unspent().count();

    println!("Balance before send: {}", before.total());
    println!("Tracked UTXOs: {utxos_before}");

    if before.trusted_spendable().to_sat() < SEND_AMOUNT + 1_000 {
        bail!("wallet does not have enough spendable regtest funds");
    }

    let destination = receive_address(&mut wallet, &mut db)?;
    let txid = send(&mut wallet, &mut db, &rpc, destination.clone(), SEND_AMOUNT)?;

    println!("Broadcast txid: {txid}");

    let raw = rpc.get_raw_transaction(&txid, None)?;
    if raw.compute_txid() != txid {
        bail!("Bitcoin Core returned a transaction with an unexpected txid");
    }

    sync_wallet(&mut wallet, &mut db, rpc.clone())?;

    let after_send = wallet.balance();
    println!("Balance after send: {after_send}");
    println!("UTXOs after send: {}", wallet.list_unspent().count());

    let external_index = wallet.next_derivation_index(KeychainKind::External);
    let internal_index = wallet.next_derivation_index(KeychainKind::Internal);

    drop(wallet);
    drop(db);

    let mut reopened_db = Connection::open(&args.db)?;
    let (reopened_wallet, _) = load_or_create_wallet(&mut reopened_db)?;

    if reopened_wallet.next_derivation_index(KeychainKind::External) != external_index
        || reopened_wallet.next_derivation_index(KeychainKind::Internal) != internal_index
    {
        bail!("wallet persistence verification failed");
    }

    println!("Persistence check: PASS");
    println!("Transaction verification: PASS ({txid})");
    println!("Demo complete.");

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Demo => demo(&args)?,

        Command::Address { change } => {
            let mut db = Connection::open(&args.db)?;
            let (mut wallet, _) = load_or_create_wallet(&mut db)?;

            let address = if change {
                change_address(&mut wallet, &mut db)?
            } else {
                receive_address(&mut wallet, &mut db)?
            };

            println!("{address}");
        }

        Command::Sync => {
            let rpc = Arc::new(rpc_client(&args)?);
            let mut db = Connection::open(&args.db)?;
            let (mut wallet, _) = load_or_create_wallet(&mut db)?;
            sync_wallet(&mut wallet, &mut db, rpc)?;
            println!("Balance: {}", wallet.balance());
            println!("UTXOs: {}", wallet.list_unspent().count());
        }

        Command::Send {
            ref address,
            amount_sat,
        } => {
            let rpc = rpc_client(&args)?;
            let destination = Address::from_str(address)
                .context("invalid destination address")?
                .require_network(NETWORK)
                .context("destination address is not for regtest")?;

            let mut db = Connection::open(&args.db)?;
            let (mut wallet, _) = load_or_create_wallet(&mut db)?;
            let txid = send(&mut wallet, &mut db, &rpc, destination, amount_sat)?;
            println!("Broadcast txid: {txid}");
        }
    }

    Ok(())
}

trait OptionalRow<T> {
    fn optional(self) -> Result<Option<T>, bdk_wallet::rusqlite::Error>;
}

impl<T> OptionalRow<T> for Result<T, bdk_wallet::rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, bdk_wallet::rusqlite::Error> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(bdk_wallet::rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error),
        }
    }
}
