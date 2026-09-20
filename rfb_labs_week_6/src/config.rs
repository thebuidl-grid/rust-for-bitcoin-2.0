use std::env;

use anyhow::{Context, Result};
use bitcoin::Network;

/// All the runtime configuration this wallet needs, loaded from environment
/// variables (via a local `.env` file) rather than hardcoded — the assignment
/// explicitly requires no committed secrets or seed material.
#[derive(Debug, Clone)]
pub struct Config {
    pub rpc_url: String,
    pub rpc_user: String,
    pub rpc_password: String,
    pub network: Network,
}

impl Config {
    /// Load config from environment variables, reading `.env` first if present.
    pub fn from_env() -> Result<Self> {
        // `.ok()` here deliberately swallows the "file not found" case: `.env`
        // is a local dev convenience, not a hard requirement — in CI or a
        // container, the same variables might be injected directly into the
        // environment instead of via a file.
        dotenvy::dotenv().ok();

        // Each `.context(...)` turns a bare "environment variable not found"
        // error into a message that actually tells the user what to fix.
        let rpc_url = env::var("BITCOIN_RPC_URL")
            .context("BITCOIN_RPC_URL must be set (see .env.example)")?;
        let rpc_user = env::var("BITCOIN_RPC_USER")
            .context("BITCOIN_RPC_USER must be set (see .env.example)")?;
        let rpc_password = env::var("BITCOIN_RPC_PASSWORD")
            .context("BITCOIN_RPC_PASSWORD must be set (see .env.example)")?;
        let network_str = env::var("BITCOIN_NETWORK")
            .context("BITCOIN_NETWORK must be set (see .env.example)")?;

        // Parse the network string into bitcoin's actual Network enum here,
        // once, so every other part of the app just works with `Network`
        // instead of re-parsing or comparing raw strings later.
        let network = match network_str.to_lowercase().as_str() {
            "regtest" => Network::Regtest,
            "testnet" => Network::Testnet,
            "signet" => Network::Signet,
            other => anyhow::bail!(
                "unsupported BITCOIN_NETWORK '{other}' — use regtest, testnet, or signet"
            ),
        };

        Ok(Self {
            rpc_url,
            rpc_user,
            rpc_password,
            network,
        })
    }
}
