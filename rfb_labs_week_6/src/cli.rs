use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "rfb-wallet", about = "A minimal regtest Bitcoin wallet")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Sync wallet state against the configured Bitcoin Core node.
    Sync,
    /// Print the current wallet balance breakdown.
    Balance,
    /// Reveal the next unused address.
    Address {
        /// Reveal an internal (change) address instead of an external (receive) one.
        #[arg(long)]
        internal: bool,
    },
    /// Build, sign, and broadcast a transaction.
    Send {
        /// Destination address.
        address: String,
        /// Amount to send, in BTC (e.g. 0.5).
        amount_btc: f64,
    },
}
