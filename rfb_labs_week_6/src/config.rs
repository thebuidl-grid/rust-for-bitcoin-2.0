//! CLI arguments and `.env`-backed configuration.
//!
//! Every field can come from an environment variable (loaded from `.env` via
//! `dotenvy` in `main`) or a `--flag` override. Nothing secret is hardcoded:
//! the BIP39 mnemonic is either read from `MNEMONIC` in `.env` or generated
//! fresh by `init` and appended there.

use std::path::PathBuf;

use bdk_wallet::bitcoin::Network;
use clap::{Args, Parser, Subcommand, ValueEnum};

/// A minimal regtest/testnet Bitcoin wallet built on `bdk_wallet` +
/// `bitcoincore-rpc`.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub config: Config,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Generate (or load) the wallet's mnemonic and create its local descriptor wallet + SQLite store.
    Init,
    /// Reveal and print the next unused external (receive) address.
    Address,
    /// Reveal and print the next unused internal (change) address.
    ChangeAddress,
    /// Sync local wallet state against the configured Bitcoin Core node.
    Sync,
    /// Sync, then print the wallet's balance breakdown.
    Balance,
    /// Sync, then list every UTXO the wallet knows about.
    Utxos,
    /// Sync, then build, sign, and broadcast a transaction paying `amount_sats` to `address`.
    Send {
        /// Destination address (must match the configured network).
        address: String,
        /// Amount to send, in satoshis.
        amount_sats: u64,
    },
}

#[derive(Args, Debug, Clone)]
pub struct Config {
    /// Bitcoin network: bitcoin, testnet, signet, or regtest.
    #[arg(env = "BITCOIN_NETWORK", long, default_value = "regtest")]
    pub network: Network,

    /// BIP39 mnemonic. Set by `init` (in `.env`, never committed) or supplied directly.
    #[arg(env = "MNEMONIC", long)]
    pub mnemonic: Option<String>,

    /// Optional BIP39 passphrase ("25th word").
    #[arg(env = "PASSPHRASE", long, default_value = "")]
    pub passphrase: String,

    /// Output descriptor script type to derive the wallet from.
    #[arg(env = "DESCRIPTOR_KIND", long, value_enum, default_value = "wpkh")]
    pub descriptor_kind: DescriptorKind,

    /// BIP32 account index (the `account'` level of the derivation path).
    #[arg(env = "ACCOUNT", long, default_value_t = 0)]
    pub account: u32,

    /// Path to the wallet's local SQLite state file.
    #[arg(env = "WALLET_DB", long, default_value = "wallet.sqlite")]
    pub db_path: PathBuf,

    /// Bitcoin Core RPC host:port.
    #[arg(env = "RPC_URL", long, default_value = "127.0.0.1:18443")]
    pub rpc_url: String,

    /// Bitcoin Core RPC cookie file (used instead of user/pass if set).
    #[arg(env = "RPC_COOKIE", long)]
    pub rpc_cookie: Option<PathBuf>,

    /// Bitcoin Core RPC username.
    #[arg(env = "RPC_USER", long)]
    pub rpc_user: Option<String>,

    /// Bitcoin Core RPC password.
    #[arg(env = "RPC_PASS", long)]
    pub rpc_pass: Option<String>,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescriptorKind {
    /// BIP84 native SegWit, `wpkh(...)`.
    Wpkh,
    /// BIP86 Taproot, `tr(...)`.
    Tr,
}
