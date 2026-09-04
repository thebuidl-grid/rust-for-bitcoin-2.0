/// Configuration loaded from the environment / .env file.
///
/// Never hardcode credentials here.  All values come from environment
/// variables (populated by a `.env` file via the `dotenv` crate, or set
/// directly in the shell).
use anyhow::{Context, Result};
use std::env;

pub struct Config {
    /// Full Bitcoin Core RPC URL, e.g. `http://127.0.0.1:18443`
    pub rpc_url: String,
    /// RPC username
    pub rpc_user: String,
    /// RPC password
    pub rpc_pass: String,
    /// Path to the SQLite wallet database file
    pub wallet_db: String,
    /// Optional BIP-39 mnemonic for wallet import/recovery.
    /// `None` → generate a fresh wallet on first run.
    pub mnemonic: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            rpc_url: env::var("RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:18443".into()),
            rpc_user: env::var("RPC_USER").context("RPC_USER environment variable not set")?,
            rpc_pass: env::var("RPC_PASS").context("RPC_PASS environment variable not set")?,
            wallet_db: env::var("WALLET_DB").unwrap_or_else(|_| "wallet.sqlite".into()),
            mnemonic: env::var("MNEMONIC").ok(),
        })
    }
}
