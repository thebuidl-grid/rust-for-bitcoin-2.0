use std::path::PathBuf;

use bitcoin::Network;

use crate::error::{WalletError, WalletResult};

#[derive(Clone, Debug)]
pub struct WalletConfig {
    pub network: Network,
    pub data_dir: PathBuf,
    pub database_path: PathBuf,
    pub rpc: RpcConfig,
}

#[derive(Clone, Debug)]
pub struct RpcConfig {
    pub url: String,
    pub user: Option<String>,
    pub password: Option<String>,
}

impl WalletConfig {
    pub fn new(network: Network, data_dir: PathBuf, rpc: RpcConfig) -> WalletResult<Self> {
        rpc.validate()?;

        Ok(Self {
            network,
            database_path: data_dir.join("wallet.sqlite3"),
            data_dir,
            rpc,
        })
    }
}

impl RpcConfig {
    pub fn new(url: String, user: Option<String>, password: Option<String>) -> Self {
        Self {
            url,
            user,
            password,
        }
    }

    fn validate(&self) -> WalletResult<()> {
        if self.user.is_some() != self.password.is_some() {
            return Err(WalletError::Config(
                "RPC username and password must be supplied together".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::RpcConfig;

    #[test]
    fn rejects_partial_rpc_credentials() {
        let config = RpcConfig::new("http://localhost:18443".into(), Some("user".into()), None);

        assert!(config.validate().is_err());
    }
}
