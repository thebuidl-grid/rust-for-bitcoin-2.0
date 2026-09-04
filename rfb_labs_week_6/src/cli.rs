use std::str::FromStr;
use std::sync::Arc;

use bdk_wallet::PersistedWallet;
use bitcoin::{Address, Amount, FeeRate};
use bitcoincore_rpc::Client;
use clap::{Parser, Subcommand};

use crate::config::Config;
use crate::wallet::WalletDb;
use crate::{keys, node, persist, tx, wallet};

/// A minimal regtest Bitcoin wallet built on rust-bitcoin, bitcoincore-rpc,
/// and BDK.
#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Sync against the node and print the wallet balance and UTXO count.
    Balance,
    /// Reveal and print the next address.
    Address {
        /// Reveal a change (internal) address instead of a receive (external) one.
        #[arg(long)]
        internal: bool,
    },
    /// Sync the wallet's view of the chain against the configured node.
    Sync,
    /// Build, sign, and broadcast a transaction.
    Send {
        /// Recipient address.
        #[arg(long)]
        to: String,
        /// Amount to send, in BTC.
        #[arg(long)]
        amount: f64,
        /// Fee rate, in sat/vB.
        #[arg(long, default_value_t = 2)]
        fee_rate: u64,
    },
    /// Regtest only: mine blocks to a fresh receive address.
    Fund {
        /// Blocks to mine. Defaults to enough for one matured coinbase.
        #[arg(long, default_value_t = node::COINBASE_MATURITY + 1)]
        blocks: u64,
    },
    /// Regtest only: full create → fund → sync → send → confirm
    /// walkthrough, used to verify the wallet end-to-end.
    Demo,
}

pub fn run(command: Command, config: &Config) -> anyhow::Result<()> {
    let mnemonic = keys::load_or_generate_mnemonic(config.mnemonic.as_deref())?;
    let descriptors = keys::descriptors_from_mnemonic(&mnemonic, config.network.into())?;
    println!("external descriptor (public): {}", descriptors.external_public);
    println!("internal descriptor (public): {}", descriptors.internal_public);

    let mut db = persist::open_db(&config.db_path)?;
    let mut w = wallet::open_or_create_wallet(
        descriptors.external,
        descriptors.internal,
        config.network,
        &mut db,
    )?;

    match command {
        Command::Balance => cmd_balance(&mut w, config, &mut db),
        Command::Address { internal } => cmd_address(&mut w, &mut db, internal),
        Command::Sync => cmd_sync(&mut w, config, &mut db),
        Command::Send { to, amount, fee_rate } => {
            cmd_send(&mut w, config, &mut db, &to, amount, fee_rate)
        }
        Command::Fund { blocks } => cmd_fund(&mut w, config, &mut db, blocks),
        Command::Demo => cmd_demo(&mut w, config, &mut db),
    }
}

fn build_client(config: &Config) -> anyhow::Result<Arc<Client>> {
    Ok(Arc::new(node::build_rpc_client(&config.rpc_url, &config.rpc_auth)?))
}

fn cmd_address(
    w: &mut PersistedWallet<WalletDb>,
    db: &mut WalletDb,
    internal: bool,
) -> anyhow::Result<()> {
    let address = if internal {
        wallet::new_change_address(w)
    } else {
        wallet::new_receive_address(w)
    };
    w.persist(db)?;
    println!("{}", address.address);
    Ok(())
}

fn cmd_sync(
    w: &mut PersistedWallet<WalletDb>,
    config: &Config,
    db: &mut WalletDb,
) -> anyhow::Result<()> {
    let client = build_client(config)?;
    node::sync_wallet(w, client, db)?;
    println!("synced");
    Ok(())
}

fn cmd_balance(
    w: &mut PersistedWallet<WalletDb>,
    config: &Config,
    db: &mut WalletDb,
) -> anyhow::Result<()> {
    let client = build_client(config)?;
    node::sync_wallet(w, client, db)?;
    let balance = wallet::get_balance(w);
    let utxos = wallet::list_utxos(w);
    println!(
        "total={} confirmed={} trusted_pending={} untrusted_pending={} immature={}",
        balance.total(),
        balance.confirmed,
        balance.trusted_pending,
        balance.untrusted_pending,
        balance.immature
    );
    println!("utxos: {}", utxos.len());
    Ok(())
}

fn cmd_send(
    w: &mut PersistedWallet<WalletDb>,
    config: &Config,
    db: &mut WalletDb,
    to: &str,
    amount_btc: f64,
    fee_rate_sat_vb: u64,
) -> anyhow::Result<()> {
    let recipient = Address::from_str(to)?.require_network(config.network)?;
    let amount = Amount::from_btc(amount_btc)?;
    let fee_rate = FeeRate::from_sat_per_vb(fee_rate_sat_vb)
        .ok_or_else(|| anyhow::anyhow!("fee rate {fee_rate_sat_vb} sat/vB is out of range"))?;

    let client = build_client(config)?;
    let txid = tx::send(w, &client, db, &recipient, amount, fee_rate)?;
    println!("broadcast txid: {txid}");
    Ok(())
}

fn cmd_fund(
    w: &mut PersistedWallet<WalletDb>,
    config: &Config,
    db: &mut WalletDb,
    blocks: u64,
) -> anyhow::Result<()> {
    let client = build_client(config)?;
    let (chain, _) = node::chain_info(&client)?;
    if chain != "regtest" {
        anyhow::bail!("fund is only available on regtest (connected chain: {chain})");
    }
    let address = wallet::new_receive_address(w);
    w.persist(db)?;
    let mined = node::fund_wallet_regtest(&client, &address.address, blocks)?;
    println!("mined {} blocks to {}", mined.len(), address.address);
    Ok(())
}

fn cmd_demo(
    w: &mut PersistedWallet<WalletDb>,
    config: &Config,
    db: &mut WalletDb,
) -> anyhow::Result<()> {
    let receive = wallet::new_receive_address(w);
    let change = wallet::new_change_address(w);
    w.persist(db)?;
    println!("receive address (external): {}", receive.address);
    println!("change address (internal):  {}", change.address);

    let client = build_client(config)?;
    let (chain, blocks) = node::chain_info(&client)?;
    println!("connected to node: chain={chain} blocks={blocks}");

    if chain != "regtest" {
        anyhow::bail!("demo is only available on regtest (connected chain: {chain})");
    }

    let mined = node::fund_wallet_regtest(&client, &receive.address, node::COINBASE_MATURITY + 1)?;
    println!("mined {} blocks to {}", mined.len(), receive.address);

    node::sync_wallet(w, Arc::clone(&client), db)?;
    let balance = wallet::get_balance(w);
    let utxos = wallet::list_utxos(w);
    println!(
        "balance: total={} confirmed={} trusted_pending={}",
        balance.total(),
        balance.confirmed,
        balance.trusted_pending
    );
    println!("utxos: {}", utxos.len());

    if balance.confirmed > Amount::ZERO {
        let send_to = wallet::new_receive_address(w);
        w.persist(db)?;

        // Hardcoded demo values, not user input.
        let demo_amount = Amount::from_btc(1.0).expect("1.0 BTC is always representable");
        let demo_fee_rate =
            FeeRate::from_sat_per_vb(2).expect("2 sat/vB is always representable");
        let txid = tx::send(w, &client, db, &send_to.address, demo_amount, demo_fee_rate)?;
        println!("broadcast txid: {txid}");

        node::fund_wallet_regtest(&client, &receive.address, 1)?;
        node::sync_wallet(w, Arc::clone(&client), db)?;
        let balance = wallet::get_balance(w);
        println!(
            "balance after 1 confirmation: total={} confirmed={}",
            balance.total(),
            balance.confirmed
        );
    } else {
        println!("no confirmed balance yet — skipping send demo");
    }

    Ok(())
}
