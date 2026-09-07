mod bitrpc;
mod keys;
mod node;
mod tx;
mod wallet;

use std::path::Path;
use anyhow::Result;
use bdk_wallet::bitcoin::{Address, Amount, Network};
use bdk_wallet::KeychainKind;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bitcoin-wallet", about = "A Bitcoin wallet (BDK + BitRPC)")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Show a new receiving address.
    NewAddress,
    /// Sync with the node and print the current balance.
    Balance,
    /// Build, sign, and broadcast a transaction. THIS IS MAINNET -- moves
    /// real BTC. Requires --confirm to actually broadcast.
    Send {
        /// Destination address.
        address: String,
        /// Amount in satoshis.
        amount_sats: u64,
        /// Required flag to actually broadcast (safety check against
        /// accidental mainnet sends).
        #[arg(long)]
        confirm: bool,
    },
}

fn main() -> Result<()> {

    let network = Network::Bitcoin;

    let cli = Cli::parse();

    let master_xprv = keys::load_or_generate_master_key(".env", network)?;
    let mut conn = wallet::open_db(Path::new("wallet.sqlite"))?;
    let mut w = wallet::load_or_create_wallet(&mut conn, master_xprv, network)?;

    match cli.command {
        Command::NewAddress => {
            let addr = w.reveal_next_address(KeychainKind::External);
            println!("{}", addr.address);
            wallet::save(&mut w, &mut conn)?;
        }
        Command::Balance => {
            let client = bitrpc::BitRpcClient::from_env()?;
            node::sync_wallet(&mut w, &client)?;
            wallet::save(&mut w, &mut conn)?;

            let balance = w.balance();
            println!("confirmed:   {}", balance.confirmed);
            println!("untrusted:   {}", balance.untrusted_pending);
            println!("trusted pnd: {}", balance.trusted_pending);
            println!("total:       {}", balance.total());
        }
        Command::Send { address, amount_sats, confirm } => {
            let client = bitrpc::BitRpcClient::from_env()?;
            node::sync_wallet(&mut w, &client)?;

            let dest = address.parse::<Address<_>>()?.require_network(network)?;
            let amount = Amount::from_sat(amount_sats);

            if !confirm {
                println!("This is MAINNET. About to send {amount} to {dest}.");
                println!("Re-run with --confirm to actually broadcast. Nothing was sent.");
                return Ok(());
            }

            let txid = tx::send(&mut w, &client, &dest, amount)?;
            wallet::save(&mut w, &mut conn)?;
            println!("broadcast txid: {txid}");
        }
    }

    Ok(())
}