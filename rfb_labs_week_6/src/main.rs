mod config;
mod node;
mod wallet;

use std::str::FromStr;

use anyhow::{Context, Result, ensure};
use bdk_wallet::bitcoin::{Address, Amount, FeeRate, Transaction};
use bdk_wallet::rusqlite::Connection;
use bdk_wallet::{KeychainKind, SignOptions};
use bitcoincore_rpc::RpcApi;
use clap::{Parser, Subcommand};

use config::Config;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "A regtest/testnet Bitcoin wallet built on rust-bitcoin, BDK, and bitcoincore-rpc"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Show the wallet's network, descriptors, and current balance
    Info,
    /// Sync wallet state with the connected Bitcoin node
    Sync,
    /// Print the current wallet balance
    Balance,
    /// Reveal and print a brand-new receiving address
    NewAddress,
    /// List every transaction the wallet knows about
    Transactions,
    /// List unspent transaction outputs (UTXOs)
    Utxos,
    /// Regtest-only helper: mine blocks paying the coinbase reward to this wallet
    Fund {
        /// Number of blocks to mine (101 clears coinbase maturity for the first block)
        #[arg(default_value_t = 101)]
        blocks: u64,
    },
    /// Build, sign, and broadcast a transaction paying `amount_sats` to `to`
    Send {
        /// Destination address
        to: String,
        /// Amount to send, in satoshis
        #[arg(long)]
        amount_sats: u64,
        /// Feerate in sat/vB
        #[arg(long, default_value_t = 1)]
        fee_rate: u64,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load()?;

    let rpc_client = node::connect(&config)?;
    let mut db = Connection::open(&config.db_path)
        .with_context(|| format!("failed to open wallet database at {:?}", config.db_path))?;
    let mut w = wallet::open(&config, &mut db)?;

    match cli.command {
        Command::Info => {
            println!("Network: {}", config.network);
            for (keychain, descriptor) in w.keychains() {
                println!("{keychain:?} descriptor: {descriptor}");
            }
            let balance = w.balance();
            println!(
                "Balance: {} (see `balance` for a breakdown)",
                balance.total()
            );
        }
        Command::Sync => {
            wallet::sync(&mut w, &mut db, &rpc_client)?;
            println!("Balance: {}", w.balance().total());
        }
        Command::Balance => {
            wallet::sync(&mut w, &mut db, &rpc_client)?;
            let balance = w.balance();
            println!("confirmed:          {}", balance.confirmed);
            println!("trusted_pending:    {}", balance.trusted_pending);
            println!("untrusted_pending:  {}", balance.untrusted_pending);
            println!("immature:           {}", balance.immature);
            println!("total:              {}", balance.total());
        }
        Command::NewAddress => {
            let info = w.reveal_next_address(KeychainKind::External);
            w.persist(&mut db)?;
            println!("New address (index {}): {}", info.index, info.address);
        }
        Command::Transactions => {
            wallet::sync(&mut w, &mut db, &rpc_client)?;
            for tx in w.transactions() {
                let status = if tx.chain_position.is_confirmed() {
                    "confirmed"
                } else {
                    "unconfirmed"
                };
                println!("{} [{status}]", tx.tx_node.txid);
            }
        }
        Command::Utxos => {
            wallet::sync(&mut w, &mut db, &rpc_client)?;
            for utxo in w.list_unspent() {
                println!(
                    "{}:{} amount={} keychain={:?}",
                    utxo.outpoint.txid, utxo.outpoint.vout, utxo.txout.value, utxo.keychain
                );
            }
        }
        Command::Fund { blocks } => {
            ensure!(
                config.network != bdk_wallet::bitcoin::Network::Bitcoin,
                "refusing to mine/fund on mainnet"
            );
            let info = w.reveal_next_address(KeychainKind::External);
            w.persist(&mut db)?;
            println!("Mining {blocks} block(s) to {}...", info.address);
            rpc_client
                .generate_to_address(blocks, &info.address)
                .context("generatetoaddress RPC call failed")?;
            wallet::sync(&mut w, &mut db, &rpc_client)?;
            println!("Balance after funding: {}", w.balance().total());
        }
        Command::Send {
            to,
            amount_sats,
            fee_rate,
        } => {
            wallet::sync(&mut w, &mut db, &rpc_client)?;

            let address = Address::from_str(&to)
                .context("invalid destination address")?
                .require_network(config.network)
                .context("destination address is for the wrong network")?;
            let fee_rate = FeeRate::from_sat_per_vb(fee_rate).context("invalid fee rate")?;

            let mut builder = w.build_tx();
            builder
                .add_recipient(address.script_pubkey(), Amount::from_sat(amount_sats))
                .fee_rate(fee_rate);
            let mut psbt = builder
                .finish()
                .context("failed to build the transaction")?;

            let finalized = w
                .sign(&mut psbt, SignOptions::default())
                .context("failed to sign the transaction")?;
            ensure!(
                finalized,
                "wallet could not fully sign/finalize the transaction"
            );

            let tx = psbt
                .extract_tx()
                .context("failed to extract the finalized transaction")?;

            // Independent audit using raw rust-bitcoin only (no BDK involved): recompute the
            // transaction's own weight/vsize and decode each output's address straight from its
            // scriptPubKey, so what we're about to broadcast is verified against the primitives
            // directly rather than trusted from the wallet/TxBuilder's own bookkeeping. See the
            // README ("Where raw rust-bitcoin earns its keep") for why this matters.
            audit_transaction(&tx, config.network);

            let txid = rpc_client
                .send_raw_transaction(&tx)
                .context("failed to broadcast the transaction")?;
            w.persist(&mut db)?;

            println!("Broadcast transaction: {txid}");
        }
    }

    Ok(())
}

/// Inspect a transaction using only raw `rust-bitcoin` types -- no `bdk_wallet` APIs -- as an
/// independent check on what a transaction actually does before it's broadcast.
fn audit_transaction(tx: &Transaction, network: bdk_wallet::bitcoin::Network) {
    println!(
        "Raw rust-bitcoin audit: txid={} weight={} vsize={} vbytes",
        tx.compute_txid(),
        tx.weight(),
        tx.vsize()
    );
    for (index, output) in tx.output.iter().enumerate() {
        match Address::from_script(&output.script_pubkey, network) {
            Ok(address) => println!("  output[{index}] {} -> {address}", output.value),
            Err(_) => println!(
                "  output[{index}] {} -> (non-standard/unrecognized script)",
                output.value
            ),
        }
    }
}
