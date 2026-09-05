//! Connection to Bitcoin Core.
//!
//! This is the wallet's only link to the outside world: everything it knows about the chain
//! (blocks, mempool, the ability to broadcast) comes through this RPC client. Configuration is
//! read from environment variables (normally loaded from a local `.env` by `main.rs`) so no
//! credentials are hardcoded in source.

use bitcoincore_rpc::{Auth, Client};

use crate::error::{Result, WalletError};

pub struct RpcConfig {
    pub url: String,
    pub user: String,
    pub password: String,
}

impl RpcConfig {
    /// Reads `BITCOIN_RPC_URL` / `BITCOIN_RPC_USER` / `BITCOIN_RPC_PASSWORD` from the process
    /// environment. Call `dotenvy::dotenv()` before this if you want `.env` to populate them.
    pub fn from_env() -> Result<Self> {
        let url = std::env::var("BITCOIN_RPC_URL")
            .map_err(|_| WalletError::App("BITCOIN_RPC_URL is not set (check your .env)".into()))?;
        let user = std::env::var("BITCOIN_RPC_USER")
            .map_err(|_| WalletError::App("BITCOIN_RPC_USER is not set (check your .env)".into()))?;
        let password = std::env::var("BITCOIN_RPC_PASSWORD").map_err(|_| {
            WalletError::App("BITCOIN_RPC_PASSWORD is not set (check your .env)".into())
        })?;
        Ok(Self { url, user, password })
    }

    pub fn client(&self) -> Result<Client> {
        let client = Client::new(
            &self.url,
            Auth::UserPass(self.user.clone(), self.password.clone()),
        )
        .map_err(WalletError::from)?;
        Ok(client)
    }
}
