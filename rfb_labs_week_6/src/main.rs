use anyhow::{Result, bail};
use bdk_bitcoind_rpc::{Emitter, NO_EXPECTED_MEMPOOL_TXS};
use bdk_wallet::bitcoin::bip32::Xpriv;
use bdk_wallet::bitcoin::{Address, Amount, Network};
use bdk_wallet::rusqlite::Connection;
use bdk_wallet::{KeychainKind, PersistedWallet, SignOptions, Wallet};
use bitcoincore_rpc::{Auth, Client, RpcApi};
use dotenvy::dotenv;
use rand::RngCore;
use std::env;

const WALLET_DB: &str = "wallet.sqlite";

fn generate_key() -> Result<()> {
    let mut seed = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut seed);

    let xprv = Xpriv::new_master(Network::Regtest, &seed)?;

    println!("Generated disposable regtest xprv:");
    println!("{xprv}");
    println!();
    println!("Add it to .env as:");
    println!("WALLET_XPRV={xprv}");

    Ok(())
}

fn connect_rpc() -> Result<Client> {
    let rpc_url = env::var("RPC_URL")?;
    let rpc_user = env::var("RPC_USER")?;
    let rpc_password = env::var("RPC_PASSWORD")?;

    Ok(Client::new(
        &rpc_url,
        Auth::UserPass(rpc_user, rpc_password),
    )?)
}

fn load_wallet(conn: &mut Connection) -> Result<PersistedWallet<Connection>> {
    let xprv = env::var("WALLET_XPRV").expect("WALLET_XPRV must be set in .env");

    let external_descriptor = format!("wpkh({xprv}/84'/1'/0'/0/*)");
    let internal_descriptor = format!("wpkh({xprv}/84'/1'/0'/1/*)");

    let wallet = match Wallet::load()
        .descriptor(KeychainKind::External, Some(external_descriptor.clone()))
        .descriptor(KeychainKind::Internal, Some(internal_descriptor.clone()))
        .extract_keys()
        .check_network(Network::Regtest)
        .load_wallet(conn)?
    {
        Some(wallet) => {
            println!("Loaded existing wallet.");
            wallet
        }

        None => {
            println!("Creating new wallet...");

            Wallet::create(external_descriptor, internal_descriptor)
                .network(Network::Regtest)
                .create_wallet(conn)?
        }
    };

    Ok(wallet)
}

fn sync_wallet(wallet: &mut PersistedWallet<Connection>, rpc: &Client) -> Result<()> {
    let wallet_tip = wallet.latest_checkpoint();

    let mut emitter = Emitter::new(
        rpc,
        wallet_tip.clone(),
        wallet_tip.height(),
        NO_EXPECTED_MEMPOOL_TXS,
    );

    println!("Syncing wallet...");

    while let Some(block) = emitter.next_block()? {
        wallet.apply_block_connected_to(
            &block.block,
            block.block_height(),
            block.connected_to(),
        )?;
    }

    let mempool = emitter.mempool()?.update;
    wallet.apply_unconfirmed_txs(mempool);

    println!("Wallet synced.");

    Ok(())
}

fn show_wallet(wallet: &mut PersistedWallet<Connection>, conn: &mut Connection) -> Result<()> {
    let balance = wallet.balance();

    println!();
    println!("=== Wallet ===");
    println!("Network: regtest");
    println!("Balance: {}", balance.total());

    let receive = wallet.reveal_next_address(KeychainKind::External);
    let change = wallet.reveal_next_address(KeychainKind::Internal);

    println!("Receive address: {}", receive.address);
    println!("Change address:  {}", change.address);

    println!();
    println!("=== UTXOs ===");

    for utxo in wallet.list_unspent() {
        println!(
            "{}:{} — {}",
            utxo.outpoint.txid, utxo.outpoint.vout, utxo.txout.value
        );
    }

    wallet.persist(conn)?;

    Ok(())
}

fn send(
    wallet: &mut PersistedWallet<Connection>,
    conn: &mut Connection,
    rpc: &Client,
    recipient: &str,
    amount_btc: f64,
) -> Result<()> {
    if amount_btc <= 0.0 {
        bail!("Amount must be greater than zero.");
    }

    let recipient_address = recipient
        .parse::<Address<_>>()?
        .require_network(Network::Regtest)?;

    let mut tx_builder = wallet.build_tx();

    tx_builder.add_recipient(
        recipient_address.script_pubkey(),
        Amount::from_btc(amount_btc)?,
    );

    let mut psbt = tx_builder.finish()?;

    // Persist wallet changes such as the reserved change output.
    wallet.persist(conn)?;

    println!();
    println!("=== Transaction ===");
    println!("PSBT created successfully.");
    println!("Inputs: {}", psbt.inputs.len());
    println!("Outputs: {}", psbt.outputs.len());

    let finalized = wallet.sign(&mut psbt, SignOptions::default())?;

    println!("Transaction signed: {finalized}");

    if !finalized {
        bail!("Transaction could not be fully signed.");
    }

    let tx = psbt.extract_tx()?;
    let txid = rpc.send_raw_transaction(&tx)?;

    wallet.persist(conn)?;

    println!("Transaction broadcast successfully!");
    println!("TXID: {txid}");

    Ok(())
}

fn main() -> Result<()> {
    dotenv().ok();

    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(String::as_str).unwrap_or("status");

    // Key generation does not require a running Bitcoin node.
    if command == "generate-key" {
        return generate_key();
    }

    let rpc = connect_rpc()?;

    let blockchain_info = rpc.get_blockchain_info()?;

    println!(
        "Connected to Bitcoin Core: {} at height {}",
        blockchain_info.chain, blockchain_info.blocks
    );

    let mut conn = Connection::open(WALLET_DB)?;
    let mut wallet = load_wallet(&mut conn)?;

    sync_wallet(&mut wallet, &rpc)?;

    match command {
        "status" => {
            show_wallet(&mut wallet, &mut conn)?;
        }

        "send" => {
            let recipient = args.get(2).ok_or_else(|| {
                anyhow::anyhow!("Usage: cargo run -- send <regtest-address> <amount>")
            })?;

            let amount = args
                .get(3)
                .ok_or_else(|| {
                    anyhow::anyhow!("Usage: cargo run -- send <regtest-address> <amount>")
                })?
                .parse::<f64>()?;

            send(&mut wallet, &mut conn, &rpc, recipient, amount)?;
        }

        _ => {
            println!();
            println!("Usage:");
            println!("  cargo run -- generate-key");
            println!("  cargo run -- status");
            println!("  cargo run -- send <regtest-address> <amount>");
        }
    }

    Ok(())
}
