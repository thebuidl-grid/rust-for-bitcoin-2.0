mod config;
mod keys;
mod node;
mod wallet;

use std::str::FromStr;

use bdk_bitcoind_rpc::bitcoincore_rpc::RpcApi;
use bdk_wallet::bitcoin::{Address, Amount, FeeRate};
use bdk_wallet::{KeychainKind, SignOptions};
use clap::{Parser, Subcommand};

use config::Config;

#[derive(Parser)]
#[command(author, version, about = "Minimal BDK regtest wallet")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create the wallet database. Generates a fresh mnemonic if MNEMONIC isn't set.
    Init,
    /// Reveal (and persist) the next receiving address.
    Address,
    /// Reveal (and persist) the next change address.
    ChangeAddress,
    /// Sync with bitcoind and print the resulting balance.
    Sync,
    /// Sync, then print the wallet's UTXOs.
    Utxos,
    /// Sync, then build, sign, and broadcast a transaction.
    Send {
        /// Destination address.
        to: String,
        /// Amount to send, in satoshis.
        amount_sats: u64,
        /// Feerate in sat/vB. Defaults to 1, fine for regtest.
        #[arg(long, default_value_t = 1)]
        fee_rate: u64,
    },
    /// Dev helper: mine `count` blocks to a wallet address (regtest only).
    Mine {
        #[arg(default_value_t = 1)]
        count: u64,
    },
}

fn open_wallet(
    config: &Config,
) -> anyhow::Result<(
    bdk_wallet::PersistedWallet<bdk_wallet::rusqlite::Connection>,
    bdk_wallet::rusqlite::Connection,
)> {
    let mnemonic = Config::mnemonic_phrase()
        .ok_or_else(|| anyhow::anyhow!("MNEMONIC is not set — run `init` first"))?;
    let (external, internal) = keys::wpkh_descriptors(&mnemonic, config.network)?;
    wallet::load_or_create(config, external, internal)
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = Config::load()?;

    match cli.command {
        Command::Init => {
            let mnemonic = match Config::mnemonic_phrase() {
                Some(existing) => {
                    println!("Using MNEMONIC already set in the environment.");
                    existing
                }
                None => {
                    let generated = keys::generate_mnemonic()?;
                    println!("Generated a new mnemonic. Add this line to .env so future");
                    println!("commands can find it — it is not saved anywhere by this tool:");
                    println!();
                    println!("MNEMONIC=\"{generated}\"");
                    println!();
                    generated
                }
            };

            let (external, internal) = keys::wpkh_descriptors(&mnemonic, config.network)?;
            println!("External descriptor: {external}");
            println!("Internal descriptor: {internal}");

            let (mut w, mut db) = wallet::load_or_create(&config, external, internal)?;
            let address = w.reveal_next_address(KeychainKind::External).address;
            w.persist(&mut db)?;
            println!("Wallet ready at {:?}", config.db_path);
            println!("First receiving address: {address}");
        }

        Command::Address => {
            let (mut w, mut db) = open_wallet(&config)?;
            let address = w.reveal_next_address(KeychainKind::External).address;
            w.persist(&mut db)?;
            println!("{address}");
        }

        Command::ChangeAddress => {
            let (mut w, mut db) = open_wallet(&config)?;
            let address = w.reveal_next_address(KeychainKind::Internal).address;
            w.persist(&mut db)?;
            println!("{address}");
        }

        Command::Sync => {
            let (mut w, mut db) = open_wallet(&config)?;
            let client = node::connect(&config)?;
            wallet::sync(&mut w, &mut db, &client)?;
            let balance = w.balance();
            println!("confirmed:          {}", balance.confirmed);
            println!("trusted pending:    {}", balance.trusted_pending);
            println!("untrusted pending:  {}", balance.untrusted_pending);
            println!("immature:           {}", balance.immature);
            println!("total:              {}", balance.total());
        }

        Command::Utxos => {
            let (mut w, mut db) = open_wallet(&config)?;
            let client = node::connect(&config)?;
            wallet::sync(&mut w, &mut db, &client)?;
            for utxo in w.list_unspent() {
                println!(
                    "{}:{}  {}  {:?}",
                    utxo.outpoint.txid, utxo.outpoint.vout, utxo.txout.value, utxo.keychain
                );
            }
        }

        Command::Send {
            to,
            amount_sats,
            fee_rate,
        } => {
            let (mut w, mut db) = open_wallet(&config)?;
            let client = node::connect(&config)?;
            wallet::sync(&mut w, &mut db, &client)?;

            let destination = Address::from_str(&to)?.require_network(w.network())?;
            let feerate = FeeRate::from_sat_per_vb(fee_rate)
                .ok_or_else(|| anyhow::anyhow!("invalid fee rate"))?;

            let mut builder = w.build_tx();
            builder
                .add_recipient(destination.script_pubkey(), Amount::from_sat(amount_sats))
                .fee_rate(feerate);
            let mut psbt = builder.finish()?;

            let finalized = w.sign(&mut psbt, SignOptions::default())?;
            anyhow::ensure!(finalized, "wallet could not fully sign the transaction");

            let tx = psbt.extract_tx()?;
            let txid = client.send_raw_transaction(&tx)?;
            w.persist(&mut db)?;
            println!("Broadcast txid: {txid}");
        }

        Command::Mine { count } => {
            let (mut w, mut db) = open_wallet(&config)?;
            let client = node::connect(&config)?;
            let address = w.reveal_next_address(KeychainKind::External).address;
            w.persist(&mut db)?;
            let hashes = client.generate_to_address(count, &address)?;
            println!("Mined {} block(s) to {address}", hashes.len());
        }
    }

    Ok(())
}
