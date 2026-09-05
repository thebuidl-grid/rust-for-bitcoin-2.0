mod db;
mod error;
mod rpc;
mod wallet;

use std::str::FromStr;
use std::sync::Arc;

use bdk_wallet::bitcoin::Network;
use bdk_wallet::KeychainKind;
use bitcoincore_rpc::RpcApi;
use clap::{Parser, Subcommand};

use error::{Result, WalletError};

#[derive(Parser)]
#[command(
    name = "wallet",
    version,
    about = "A minimal regtest Bitcoin wallet built on BDK, rust-bitcoin and bitcoincore-rpc"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create the wallet (generates a fresh seed on first run; safe to run again).
    CreateWallet,
    /// Show network, descriptor and node info.
    WalletInfo,
    /// Reveal the next receiving (external) address. Use --internal for a change address.
    NewAddress {
        #[arg(long)]
        internal: bool,
    },
    /// Sync wallet state from Bitcoin Core (blocks + mempool).
    Sync,
    /// Show the wallet's balance.
    Balance,
    /// Build, sign and broadcast a transaction.
    Send {
        /// Destination address.
        #[arg(long)]
        to: String,
        /// Amount to send, in BTC (e.g. 0.1).
        #[arg(long)]
        amount: f64,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    // Populate BITCOIN_RPC_*, NETWORK, DATABASE_URL from .env if present. Missing .env is fine
    // (e.g. vars already exported in the shell); anything else (a malformed file) we do want to
    // know about.
    match dotenvy::dotenv() {
        Ok(_) | Err(dotenvy::Error::Io(_)) => {}
        Err(e) => return Err(WalletError::App(format!("could not read .env: {e}"))),
    }

    let network_str = std::env::var("NETWORK").unwrap_or_else(|_| "regtest".to_string());
    let network = Network::from_str(&network_str)
        .map_err(|_| WalletError::App(format!("unsupported NETWORK '{network_str}'")))?;
    if network != Network::Regtest {
        eprintln!(
            "Warning: this wallet is built and tested for regtest; NETWORK={network} is untested here."
        );
    }

    let db_path = std::env::var("DATABASE_URL").unwrap_or_else(|_| "data/wallet.db".to_string());

    let cli = Cli::parse();

    let mut conn = db::open(&db_path)?;
    let mut w = wallet::open_wallet(&mut conn, network)?;

    match cli.command {
        Command::CreateWallet => {
            let ext = w.public_descriptor(KeychainKind::External).to_string();
            let int = w.public_descriptor(KeychainKind::Internal).to_string();
            println!("Wallet ready at {db_path}");
            println!("Network: {network}");
            println!("External (receiving) descriptor:\n  {ext}");
            println!("Internal (change) descriptor:\n  {int}");
        }

        Command::WalletInfo => {
            let ext = w.public_descriptor(KeychainKind::External).to_string();
            let int = w.public_descriptor(KeychainKind::Internal).to_string();
            println!("Database:  {db_path}");
            println!("Network:   {network}");
            println!("External (receiving) descriptor:\n  {ext}");
            println!("Internal (change) descriptor:\n  {int}");

            match rpc::RpcConfig::from_env().and_then(|cfg| cfg.client()) {
                Ok(client) => match client.get_blockchain_info() {
                    Ok(info) => println!(
                        "Bitcoin Core: chain={} blocks={} headers={}",
                        info.chain, info.blocks, info.headers
                    ),
                    Err(e) => println!("Bitcoin Core: connected, but request failed ({e})"),
                },
                Err(e) => println!("Bitcoin Core: not reachable ({e})"),
            }
        }

        Command::NewAddress { internal } => {
            let keychain = if internal {
                KeychainKind::Internal
            } else {
                KeychainKind::External
            };
            let address = wallet::new_address(&mut w, &mut conn, keychain)?;
            let label = if internal { "change" } else { "receiving" };
            println!("New {label} address:");
            println!("{address}");
        }

        Command::Sync => {
            let cfg = rpc::RpcConfig::from_env()?;
            let client = Arc::new(cfg.client()?);
            println!("Syncing from Bitcoin Core at {}...", cfg.url);
            let blocks = wallet::sync(&mut w, &mut conn, client)?;
            println!("Applied {blocks} new block(s).");
            print_balance(&w);
        }

        Command::Balance => {
            print_balance(&w);
        }

        Command::Send { to, amount } => {
            let cfg = rpc::RpcConfig::from_env()?;
            let client = cfg.client()?;
            let txid = wallet::send(&mut w, &mut conn, &client, &to, amount)?;
            println!();
            println!("TXID: {txid}");
        }
    }

    Ok(())
}

fn print_balance(w: &wallet::WalletHandle) {
    let balance = wallet::balance(w);
    let pending = balance.trusted_pending + balance.untrusted_pending;

    println!("Confirmed balance: {}", balance.confirmed);
    if pending != bdk_wallet::bitcoin::Amount::ZERO {
        println!("Pending balance:   {pending} (unconfirmed)");
    }
    if balance.immature != bdk_wallet::bitcoin::Amount::ZERO {
        println!(
            "Immature balance:  {} (coinbase reward(s), not yet spendable)",
            balance.immature
        );
    }
    println!("Total balance:     {}", balance.total());
    println!("Spendable now:     {}", balance.trusted_spendable());
}
