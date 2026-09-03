use std::path::PathBuf;

use bdk_wallet::bitcoin::Network;
use clap::{Parser, Subcommand, ValueEnum};

/// A minimal regtest Bitcoin wallet built on `bdk_wallet` + `bitcoincore-rpc`.
#[derive(Parser, Debug)]
#[command(name = "rfb-wallet", author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Bitcoin network to operate on.
    #[arg(
        env = "BITCOIN_NETWORK",
        long,
        default_value = "regtest",
        global = true
    )]
    pub network: Network,

    /// bitcoind RPC address (host:port, no scheme).
    #[arg(
        env = "RPC_URL",
        long,
        default_value = "127.0.0.1:18443",
        global = true
    )]
    pub rpc_url: String,

    /// Path to bitcoind's `.cookie` auth file (preferred over user/pass).
    #[arg(env = "RPC_COOKIE", long, global = true)]
    pub rpc_cookie: Option<PathBuf>,

    /// RPC username, used if no cookie file is set.
    #[arg(env = "RPC_USER", long, global = true)]
    pub rpc_user: Option<String>,

    /// RPC password, used if no cookie file is set.
    #[arg(env = "RPC_PASS", long, global = true)]
    pub rpc_pass: Option<String>,

    /// Where the wallet's local SQLite state (UTXOs, tx graph, keychain indices) lives.
    #[arg(
        env = "WALLET_DB",
        long,
        default_value = "wallet.sqlite",
        global = true
    )]
    pub db_path: PathBuf,

    /// BIP39 recovery phrase. If unset, `init` generates one and saves it to `.env`.
    #[arg(env = "MNEMONIC", long, global = true)]
    pub mnemonic: Option<String>,

    /// BIP32 account index (the `account'` level of the derivation path).
    #[arg(env = "ACCOUNT", long, default_value_t = 0, global = true)]
    pub account: u32,

    /// Which descriptor/script type to derive the wallet from.
    #[arg(env = "SCRIPT_TYPE", long, default_value = "wpkh", global = true)]
    pub script_type: ScriptType,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ScriptType {
    /// Native SegWit, BIP84 (`m/84'/.../0|1/*`).
    Wpkh,
    /// Taproot, BIP86 (`m/86'/.../0|1/*`).
    Tr,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Generate/load the wallet's key material and create its local database.
    Init,
    /// Reveal the next receiving (or change) address.
    Address {
        /// Reveal an internal (change) address instead of an external one.
        #[arg(long)]
        change: bool,
    },
    /// Sync wallet state against the configured bitcoind node.
    Sync,
    /// Sync, then print the wallet balance.
    Balance,
    /// Sync, then list all tracked UTXOs.
    Utxos,
    /// Build, sign, and broadcast a transaction.
    Send {
        /// Destination address.
        #[arg(long)]
        to: String,
        /// Amount to send, in satoshis.
        #[arg(long)]
        amount: u64,
        /// Feerate in sat/vB. Defaults to the node's relay minimum.
        #[arg(long)]
        fee_rate: Option<u64>,
    },
    /// Stretch goal: derive a CLTV-timelocked P2WSH "vault" address using raw `rust-bitcoin`,
    /// outside of anything `bdk_wallet`'s descriptor/signer machinery can express.
    VaultCreate {
        /// Block height before which the vault cannot be spent from.
        #[arg(long)]
        unlock_height: u32,
    },
    /// Stretch goal: manually build, sign (BIP143 sighash), and broadcast a spend from a
    /// CLTV-timelocked P2WSH vault created by `vault-create`.
    VaultSpend {
        /// Outpoint funding the vault, as `txid:vout`.
        #[arg(long)]
        outpoint: String,
        /// Value of that output, in satoshis.
        #[arg(long)]
        amount: u64,
        /// WIF-encoded private key printed by `vault-create`.
        #[arg(long)]
        wif: String,
        /// Hex-encoded redeem (witness) script printed by `vault-create`.
        #[arg(long)]
        redeem_script: String,
        /// Same unlock height passed to `vault-create`.
        #[arg(long)]
        unlock_height: u32,
        /// Destination address for the unlocked funds.
        #[arg(long)]
        to: String,
    },
}
