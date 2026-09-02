use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "rfb_labs_week_6",
    author = "Rust for Bitcoin 2.0",
    version = "0.1.0",
    about = "Bitcoin Wallet CLI built with rust-bitcoin, BDK, and bitcoincore-rpc"
)]
pub struct Cli {
    /// Bitcoin network (regtest, testnet, signet, bitcoin)
    #[arg(long, env = "BITCOIN_NETWORK", default_value = "regtest")]
    pub network: String,

    /// SQLite database path to store wallet state
    #[arg(long, env = "WALLET_DB_PATH", default_value = "wallet.sqlite")]
    pub db_path: PathBuf,

    /// BIP39 mnemonic phrase (or via MNEMONIC env var)
    #[arg(long, env = "MNEMONIC")]
    pub mnemonic: Option<String>,

    /// Descriptor type (wpkh or taproot)
    #[arg(long, value_enum, default_value_t = CliDescriptorType::Wpkh)]
    pub descriptor_type: CliDescriptorType,

    /// Bitcoin Core RPC URL
    #[arg(long, env = "RPC_URL", default_value = "http://127.0.0.1:18443")]
    pub rpc_url: String,

    /// Bitcoin Core RPC username
    #[arg(long, env = "RPC_USER")]
    pub rpc_user: Option<String>,

    /// Bitcoin Core RPC password
    #[arg(long, env = "RPC_PASS")]
    pub rpc_pass: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum CliDescriptorType {
    Wpkh,
    Taproot,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate a new 12-word BIP39 mnemonic phrase and display descriptors
    GenerateMnemonic,

    /// Get a new receiving (external) or change (internal) address
    Address {
        /// Use internal change keychain
        #[arg(long)]
        change: bool,
    },

    /// Display wallet balance breakdown
    Balance,

    /// List all unspent transaction outputs (UTXOs)
    Utxos,

    /// Sync wallet with connected Bitcoin Core RPC node
    Sync {
        /// Earliest block height to start syncing from
        #[arg(long, default_value_t = 0)]
        start_height: u32,
    },

    /// Construct, sign, and optionally broadcast a transaction
    Send {
        /// Destination Bitcoin address
        #[arg(long)]
        recipient: String,

        /// Amount to send in satoshis
        #[arg(long)]
        amount_sats: u64,

        /// Broadcast transaction via RPC node
        #[arg(long, default_value_t = true)]
        broadcast: bool,
    },

    /// Run demonstration of raw rust-bitcoin OP_RETURN and PSBT manipulation
    RawDemo {
        /// Custom message to embed in OP_RETURN output
        #[arg(long, default_value = "Rust for Bitcoin Week 6 Wallet")]
        message: String,
    },
}
