//! CLI + environment configuration.
//!
//! Every value here can be set on the command line or via an environment variable of the same
//! name (loaded from `.env` by `dotenvy` before `clap` parses `std::env::args`). This mirrors the
//! pattern used by BDK's own `bitcoind_rpc` example, so the same `.env` file this wallet writes on
//! `init` doubles as documentation of what each variable means.

use std::path::PathBuf;

use bdk_wallet::bitcoin::Network;
use clap::{Parser, Subcommand, ValueEnum};

/// A minimal regtest/testnet Bitcoin wallet built on `rust-bitcoin`, BDK and `bitcoincore-rpc`.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub config: Config,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug, Clone)]
pub struct Config {
    /// Bitcoin network the wallet operates on. Only regtest/testnet/signet are supported by the
    /// `send`/`mine` flows in this assignment (see the constraint in the README).
    #[arg(long, env = "BITCOIN_NETWORK", default_value = "regtest")]
    pub network: Network,

    /// Path to the SQLite database that holds the wallet's chain state, UTXOs and history.
    #[arg(long, env = "BDK_DB_PATH", default_value = "wallet.sqlite")]
    pub db_path: PathBuf,

    /// External (receiving) keychain descriptor. Contains the private key material used to sign
    /// — never commit this. Written to `.env` by `init`.
    #[arg(long, env = "DESCRIPTOR")]
    pub descriptor: Option<String>,

    /// Internal (change) keychain descriptor. Written to `.env` by `init`.
    #[arg(long, env = "CHANGE_DESCRIPTOR")]
    pub change_descriptor: Option<String>,

    /// `host:port` of the `bitcoind` JSON-RPC server.
    #[arg(long, env = "RPC_URL", default_value = "127.0.0.1:18443")]
    pub rpc_url: String,

    /// Path to bitcoind's `.cookie` auth file. Ignored if `--rpc-user`/`--rpc-pass` are set.
    #[arg(long, env = "RPC_COOKIE")]
    pub rpc_cookie: Option<PathBuf>,

    /// RPC username, for user/pass auth instead of the cookie file.
    #[arg(long, env = "RPC_USER")]
    pub rpc_user: Option<String>,

    /// RPC password, for user/pass auth instead of the cookie file.
    #[arg(long, env = "RPC_PASS")]
    pub rpc_pass: Option<String>,

    /// Height to start block sync from. `0` walks the whole regtest chain, which is fine because
    /// regtest chains are short; on a long-lived testnet node you'd bump this past the wallet's
    /// birthday to avoid rescanning from genesis.
    #[arg(long, env = "START_HEIGHT", default_value_t = 0)]
    pub start_height: u32,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Generate a fresh key and initialize the wallet from a BIP84 (or BIP86) descriptor.
    Init {
        /// Which descriptor/script type to derive the wallet from.
        #[arg(long, value_enum, default_value_t = DescriptorKind::Wpkh)]
        kind: DescriptorKind,
        /// Overwrite an existing `.env` / wallet database instead of refusing to run.
        #[arg(long)]
        force: bool,
    },
    /// Reveal (and persist) the next unused external (receiving) address.
    Address,
    /// Reveal (and persist) the next unused internal (change) address.
    ChangeAddress,
    /// Sync wallet state from the connected Bitcoin Core node and persist it.
    Sync,
    /// Sync, then print the wallet balance breakdown.
    Balance,
    /// Sync, then list the wallet's tracked UTXOs.
    Utxos,
    /// Sync, build+sign+broadcast a transaction paying `--amount` sats to `--to`.
    Send {
        /// Destination address.
        #[arg(long)]
        to: String,
        /// Amount to send, in satoshis.
        #[arg(long)]
        amount: u64,
        /// Fee rate in sat/vB. Defaults to 1 sat/vB, which is generous for regtest.
        #[arg(long, default_value_t = 1)]
        fee_rate: u64,
    },
    /// Regtest-only helper: mine `--blocks` blocks to a fresh wallet address, to fund it for
    /// testing. Refuses to run on any network other than regtest.
    Mine {
        #[arg(long, default_value_t = 101)]
        blocks: u32,
    },
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DescriptorKind {
    /// Native SegWit v0, `wpkh(...)` — BIP84 derivation path `m/84'/coin'/0'`.
    Wpkh,
    /// Taproot, `tr(...)` — BIP86 derivation path `m/86'/coin'/0'`.
    Tr,
}
