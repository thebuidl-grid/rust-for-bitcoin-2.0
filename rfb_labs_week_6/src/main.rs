mod config;
mod keys;
mod manual;
mod node;
mod walletdb;

use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};
use bdk_bitcoind_rpc::bitcoincore_rpc::RpcApi;
use bdk_wallet::bitcoin::{Address, Amount, FeeRate};
use bdk_wallet::{KeychainKind, SignOptions};
use clap::Parser;

use config::{Cli, Command};

fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    let cli = Cli::parse();

    match &cli.command {
        Command::Init => cmd_init(&cli),
        Command::Address { change } => cmd_address(&cli, *change),
        Command::Sync => cmd_sync(&cli),
        Command::Balance => cmd_balance(&cli),
        Command::Utxos => cmd_utxos(&cli),
        Command::Send {
            to,
            amount,
            fee_rate,
        } => cmd_send(&cli, to, *amount, *fee_rate),
        Command::VaultCreate { unlock_height } => manual::create(cli.network, *unlock_height),
        Command::VaultSpend {
            outpoint,
            amount,
            wif,
            redeem_script,
            unlock_height,
            to,
        } => {
            let rpc = Arc::new(node::build_rpc_client(&cli)?);
            manual::spend(
                rpc,
                cli.network,
                outpoint,
                *amount,
                wif,
                redeem_script,
                *unlock_height,
                to,
            )
        }
    }
}

fn cmd_init(cli: &Cli) -> Result<()> {
    let (mnemonic, generated) = keys::load_or_generate_mnemonic(cli.mnemonic.clone())?;
    if generated {
        println!("Generated a new mnemonic and saved it to .env (MNEMONIC=...).");
        println!("Recovery phrase: {mnemonic}");
        println!(
            "This is test key material for regtest -- back it up like real key material anyway.\n"
        );
    } else {
        println!("Loaded existing mnemonic from MNEMONIC.\n");
    }

    let descriptors =
        keys::derive_descriptors(&mnemonic, cli.network, cli.account, cli.script_type)?;
    let (mut wallet, mut db) = walletdb::open_or_create(&cli.db_path, &descriptors, cli.network)?;

    println!("Wallet database: {}", cli.db_path.display());
    println!("External descriptor: {}", descriptors.external_public);
    println!("Internal descriptor: {}", descriptors.internal_public);

    let address = wallet.reveal_next_address(KeychainKind::External);
    wallet.persist(&mut db)?;
    println!("\nFirst receiving address: {}", address.address);
    Ok(())
}

fn load_wallet(
    cli: &Cli,
) -> Result<(
    bdk_wallet::PersistedWallet<bdk_wallet::rusqlite::Connection>,
    bdk_wallet::rusqlite::Connection,
)> {
    let mnemonic = cli
        .mnemonic
        .clone()
        .context("no MNEMONIC set -- run `init` first, or set MNEMONIC in .env")?;
    let mnemonic =
        bdk_wallet::keys::bip39::Mnemonic::parse(&mnemonic).context("invalid MNEMONIC")?;
    let descriptors =
        keys::derive_descriptors(&mnemonic, cli.network, cli.account, cli.script_type)?;
    walletdb::open_or_create(&cli.db_path, &descriptors, cli.network)
}

fn cmd_address(cli: &Cli, change: bool) -> Result<()> {
    let (mut wallet, mut db) = load_wallet(cli)?;
    let keychain = if change {
        KeychainKind::Internal
    } else {
        KeychainKind::External
    };
    let address = wallet.reveal_next_address(keychain);
    wallet.persist(&mut db)?;
    println!("{}", address.address);
    Ok(())
}

fn cmd_sync(cli: &Cli) -> Result<()> {
    let (mut wallet, mut db) = load_wallet(cli)?;
    let rpc = Arc::new(node::build_rpc_client(cli)?);
    walletdb::sync(&mut wallet, &mut db, rpc)
}

fn cmd_balance(cli: &Cli) -> Result<()> {
    let (mut wallet, mut db) = load_wallet(cli)?;
    let rpc = Arc::new(node::build_rpc_client(cli)?);
    walletdb::sync(&mut wallet, &mut db, rpc)?;

    let balance = wallet.balance();
    println!("confirmed:          {}", balance.confirmed);
    println!("trusted pending:    {}", balance.trusted_pending);
    println!("untrusted pending:  {}", balance.untrusted_pending);
    println!("immature:           {}", balance.immature);
    println!("total:              {}", balance.total());
    Ok(())
}

fn cmd_utxos(cli: &Cli) -> Result<()> {
    let (mut wallet, mut db) = load_wallet(cli)?;
    let rpc = Arc::new(node::build_rpc_client(cli)?);
    walletdb::sync(&mut wallet, &mut db, rpc)?;

    let mut count = 0;
    for utxo in wallet.list_unspent() {
        count += 1;
        println!(
            "{}  {:>14}  {:?}",
            utxo.outpoint, utxo.txout.value, utxo.keychain
        );
    }
    println!("{count} utxo(s)");
    Ok(())
}

fn cmd_send(cli: &Cli, to: &str, amount_sats: u64, fee_rate: Option<u64>) -> Result<()> {
    let (mut wallet, mut db) = load_wallet(cli)?;
    let rpc = Arc::new(node::build_rpc_client(cli)?);
    walletdb::sync(&mut wallet, &mut db, rpc.clone())?;

    let to_address = Address::from_str(to)
        .context("invalid --to address")?
        .require_network(cli.network)?;

    let mut builder = wallet.build_tx();
    builder.add_recipient(to_address.script_pubkey(), Amount::from_sat(amount_sats));
    if let Some(rate) = fee_rate {
        let fee_rate = FeeRate::from_sat_per_vb(rate).context("invalid --fee-rate")?;
        builder.fee_rate(fee_rate);
    }
    let mut psbt = builder.finish().context("failed to build transaction")?;

    let finalized = wallet.sign(&mut psbt, SignOptions::default())?;
    anyhow::ensure!(finalized, "failed to sign every input of the transaction");

    let tx = psbt.extract_tx()?;
    let txid = rpc.send_raw_transaction(&tx)?;

    println!("Broadcast: {txid}");
    println!("Run `sync` again to see the pending spend reflected in balance/utxos.");
    Ok(())
}
