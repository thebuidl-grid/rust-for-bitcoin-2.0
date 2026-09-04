use std::env;
use std::fmt;
use std::path::PathBuf;

use bitcoin::Network;

use crate::error::ConfigError;

/// Mirrors `bitcoincore_rpc::Auth` so `node.rs` can convert 1:1.
#[derive(Clone)]
pub enum RpcAuthConfig {
    CookieFile(PathBuf),
    UserPass(String, String),
    None,
}

impl fmt::Debug for RpcAuthConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RpcAuthConfig::CookieFile(path) => f.debug_tuple("CookieFile").field(path).finish(),
            RpcAuthConfig::UserPass(user, _) => {
                f.debug_tuple("UserPass").field(user).field(&"<redacted>").finish()
            }
            RpcAuthConfig::None => write!(f, "None"),
        }
    }
}

pub struct Config {
    pub network: Network,
    pub rpc_url: String,
    pub rpc_auth: RpcAuthConfig,
    pub db_path: PathBuf,
    pub mnemonic: Option<String>,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("network", &self.network)
            .field("rpc_url", &self.rpc_url)
            .field("rpc_auth", &self.rpc_auth)
            .field("db_path", &self.db_path)
            .field("mnemonic", &self.mnemonic.as_ref().map(|_| "<redacted>"))
            .finish()
    }
}

impl Config {
    pub fn from_env() -> Result<Config, ConfigError> {
        dotenvy::dotenv().ok();

        let network = match env::var("BITCOIN_NETWORK")
            .unwrap_or_else(|_| "regtest".to_string())
            .as_str()
        {
            "regtest" => Network::Regtest,
            "testnet" => Network::Testnet,
            other => return Err(ConfigError::InvalidNetwork(other.to_string())),
        };

        let rpc_url = non_empty_var("RPC_URL").ok_or(ConfigError::MissingVar("RPC_URL"))?;

        let cookie_file = non_empty_var("RPC_COOKIE_FILE");
        let rpc_user = non_empty_var("RPC_USER");
        let rpc_pass = non_empty_var("RPC_PASS");
        let rpc_auth = match (cookie_file, rpc_user, rpc_pass) {
            (Some(cookie), _, _) => RpcAuthConfig::CookieFile(PathBuf::from(cookie)),
            (None, Some(user), Some(pass)) => RpcAuthConfig::UserPass(user, pass),
            _ => RpcAuthConfig::None,
        };

        let db_path = PathBuf::from(
            non_empty_var("WALLET_DB_PATH").unwrap_or_else(|| "wallet.sqlite".to_string()),
        );

        let mnemonic = non_empty_var("MNEMONIC");

        Ok(Config { network, rpc_url, rpc_auth, db_path, mnemonic })
    }
}

fn non_empty_var(key: &str) -> Option<String> {
    env::var(key).ok().filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // env::set_var/remove_var mutate global process state, so serialize
    // these tests to avoid races when cargo test runs them in parallel.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn clear_env() {
        for key in [
            "BITCOIN_NETWORK",
            "RPC_URL",
            "RPC_COOKIE_FILE",
            "RPC_USER",
            "RPC_PASS",
            "WALLET_DB_PATH",
            "MNEMONIC",
        ] {
            unsafe { env::remove_var(key) };
        }
    }

    #[test]
    fn missing_rpc_url_is_an_error() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_env();
        let err = Config::from_env().unwrap_err();
        assert!(matches!(err, ConfigError::MissingVar("RPC_URL")));
    }

    #[test]
    fn invalid_network_is_an_error() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_env();
        unsafe {
            env::set_var("BITCOIN_NETWORK", "mainnet");
            env::set_var("RPC_URL", "127.0.0.1:18443");
        }
        let err = Config::from_env().unwrap_err();
        assert!(matches!(err, ConfigError::InvalidNetwork(_)));
    }

    #[test]
    fn defaults_to_regtest_with_no_auth() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_env();
        unsafe {
            env::set_var("RPC_URL", "127.0.0.1:18443");
        }
        let config = Config::from_env().unwrap();
        assert_eq!(config.network, Network::Regtest);
        assert!(matches!(config.rpc_auth, RpcAuthConfig::None));
        assert_eq!(config.db_path, PathBuf::from("wallet.sqlite"));
    }

    #[test]
    fn debug_output_redacts_mnemonic() {
        let _guard = ENV_LOCK.lock().unwrap();
        clear_env();
        unsafe {
            env::set_var("RPC_URL", "127.0.0.1:18443");
            env::set_var("MNEMONIC", "abandon abandon abandon");
        }
        let config = Config::from_env().unwrap();
        let debug = format!("{config:?}");
        assert!(!debug.contains("abandon"));
        assert!(debug.contains("<redacted>"));
    }
}
