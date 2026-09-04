/// CLI argument and sub-command definitions (clap derive).
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "btc-wallet",
    version,
    about = "Minimal Bitcoin wallet — regtest / testnet",
    long_about = None,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Reveal the next unused receiving address.
    Address,

    /// Print the confirmed + unconfirmed balance.
    Balance,

    /// Sync wallet state against the connected Bitcoin Core node.
    Sync,

    /// Construct, sign, and broadcast a transaction.
    Send {
        /// Destination address (testnet / regtest).
        #[arg(long)]
        to: String,

        /// Amount to send in satoshis.
        #[arg(long)]
        amount_sats: u64,
    },
}
