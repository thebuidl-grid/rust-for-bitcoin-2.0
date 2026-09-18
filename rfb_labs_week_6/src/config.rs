/// Configuration management for the Bitcoin wallet.
///
/// This module handles loading configuration from:
/// - Environment variables (.env file)
/// - Command-line arguments
/// - Default values
use crate::error::{Result, WalletError};
use bitcoin::Network;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration struct containing all wallet settings.
///
/// # Fields:
/// - `bitcoin_network`: Which Bitcoin network to use (regtest for local testing)
/// - `rpc_url`: URL to connect to Bitcoin Core node (e.g., http://127.0.0.1:18332)
/// - `rpc_username`: RPC authentication username
/// - `rpc_password`: RPC authentication password
/// - `db_path`: Where to store the SQLite database with wallet data
/// - `seed_phrase`: BIP39 mnemonic seed (NOT stored in code, loaded from env)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub bitcoin_network: String,
    pub rpc_url: String,
    pub rpc_username: String,
    pub rpc_password: String,
    pub db_path: PathBuf,
    pub seed_phrase: Option<String>,
}

impl WalletConfig {
    pub fn from_env() -> Result<Self> {
        // Load .env file if it exists
        dotenv::dotenv().ok();

        let bitcoin_network =
            std::env::var("BITCOIN_NETWORK").unwrap_or_else(|_| "regtest".to_string());

        // Validate network
        match bitcoin_network.as_str() {
            "regtest" | "testnet" | "mainnet" => {}
            _ => {
                return Err(WalletError::ConfigError(
                    "BITCOIN_NETWORK must be 'regtest', 'testnet', or 'mainnet'".to_string(),
                ))
            }
        }

        let rpc_url =
            std::env::var("RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:18332".to_string());

        let rpc_username = std::env::var("RPC_USERNAME")
            .map_err(|_| WalletError::ConfigError("RPC_USERNAME not set".to_string()))?;

        let rpc_password = std::env::var("RPC_PASSWORD")
            .map_err(|_| WalletError::ConfigError("RPC_PASSWORD not set".to_string()))?;

        let db_path =
            PathBuf::from(std::env::var("DB_PATH").unwrap_or_else(|_| "./wallet.db".to_string()));

        let seed_phrase = std::env::var("SEED_PHRASE").ok();

        Ok(WalletConfig {
            bitcoin_network,
            rpc_url,
            rpc_username,
            rpc_password,
            db_path,
            seed_phrase,
        })
    }

    /// Converts the network string to bitcoin::Network enum.
    pub fn get_network(&self) -> Result<Network> {
        match self.bitcoin_network.as_str() {
            "regtest" => Ok(Network::Regtest),
            "testnet" => Ok(Network::Testnet),
            "mainnet" => Ok(Network::Bitcoin),
            _ => Err(WalletError::ConfigError(
                "Invalid network configuration".to_string(),
            )),
        }
    }

    /// Validates that all required fields are set.
    pub fn validate(&self) -> Result<()> {
        if self.rpc_username.is_empty() {
            return Err(WalletError::ConfigError(
                "RPC username is empty".to_string(),
            ));
        }

        if self.rpc_password.is_empty() {
            return Err(WalletError::ConfigError(
                "RPC password is empty".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_conversion() {
        let mut config = WalletConfig {
            bitcoin_network: "regtest".to_string(),
            rpc_url: "http://localhost:18332".to_string(),
            rpc_username: "user".to_string(),
            rpc_password: "pass".to_string(),
            db_path: PathBuf::from("test.db"),
            seed_phrase: None,
        };

        assert_eq!(config.get_network().unwrap(), Network::Regtest);

        config.bitcoin_network = "testnet".to_string();
        assert_eq!(config.get_network().unwrap(), Network::Testnet);

        config.bitcoin_network = "mainnet".to_string();
        assert_eq!(config.get_network().unwrap(), Network::Bitcoin);
    }
}
