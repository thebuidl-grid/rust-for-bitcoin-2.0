//! Runtime configuration, read from the environment (and `.env`).
//!
//! Nothing secret is compiled in and nothing secret is written to the wallet
//! database. The mnemonic lives only in the environment, which is why
//! [`Config::mnemonic`] is an `Option`: without it the wallet still opens, but
//! read-only.

use std::path::{Path, PathBuf};

use bitcoin::Network;
use bitcoincore_rpc::Auth;

use crate::error::{Error, Result};

/// Default data directory, relative to the working directory.
const DEFAULT_DB: &str = "data/wallet.sqlite";

#[derive(Debug, Clone)]
pub struct Config {
    pub network: Network,
    pub rpc_url: String,
    pub rpc_auth: RpcAuth,
    pub db_path: PathBuf,
    mnemonic: Option<String>,
    passphrase: String,
}

/// Cloneable, debuggable stand-in for [`bitcoincore_rpc::Auth`].
///
/// `Auth` is neither `Clone` nor `Debug`-friendly for our purposes, and we want
/// to be able to print how we authenticated without printing the password.
#[derive(Clone)]
pub enum RpcAuth {
    Cookie(PathBuf),
    UserPass(String, String),
}

impl std::fmt::Debug for RpcAuth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RpcAuth::Cookie(path) => write!(f, "cookie file {}", path.display()),
            RpcAuth::UserPass(user, _) => write!(f, "user/password (user `{user}`)"),
        }
    }
}

impl RpcAuth {
    pub fn into_auth(self) -> Auth {
        match self {
            RpcAuth::Cookie(path) => Auth::CookieFile(path),
            RpcAuth::UserPass(user, pass) => Auth::UserPass(user, pass),
        }
    }
}

impl Config {
    /// Load `.env` (if present) and build a configuration from the environment.
    pub fn from_env() -> Result<Self> {
        // A missing .env is fine: the variables may come from the shell.
        let _ = dotenvy::dotenv();

        let network = parse_network(&env_or("RFB_NETWORK", "regtest"))?;
        if network == Network::Bitcoin {
            return Err(Error::MainnetRefused);
        }

        let rpc_url = std::env::var("RFB_RPC_URL").unwrap_or_else(|_| default_rpc_url(network));
        let rpc_auth = resolve_auth(network)?;
        let db_path = PathBuf::from(env_or("RFB_WALLET_DB", DEFAULT_DB));

        let mnemonic = std::env::var("RFB_MNEMONIC")
            .ok()
            .map(|m| m.split_whitespace().collect::<Vec<_>>().join(" "))
            .filter(|m| !m.is_empty());
        let passphrase = std::env::var("RFB_PASSPHRASE").unwrap_or_default();

        Ok(Config {
            network,
            rpc_url,
            rpc_auth,
            db_path,
            mnemonic,
            passphrase,
        })
    }

    /// The mnemonic, if one was supplied. Absent means "watch-only".
    pub fn mnemonic(&self) -> Option<&str> {
        self.mnemonic.as_deref()
    }

    /// The mnemonic, or the error that explains how to supply one.
    pub fn require_mnemonic(&self) -> Result<&str> {
        self.mnemonic.as_deref().ok_or(Error::MissingMnemonic)
    }

    pub fn passphrase(&self) -> &str {
        &self.passphrase
    }

    /// BIP44 coin type. Every non-mainnet chain uses 1.
    pub fn coin_type(&self) -> u32 {
        match self.network {
            Network::Bitcoin => 0,
            _ => 1,
        }
    }

    /// Create the parent directory of the wallet database if it does not exist.
    pub fn ensure_db_dir(&self) -> Result<()> {
        if let Some(parent) = self.db_path.parent()
            && !parent.as_os_str().is_empty()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent)?;
        }
        Ok(())
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn parse_network(value: &str) -> Result<Network> {
    match value.trim().to_ascii_lowercase().as_str() {
        "regtest" => Ok(Network::Regtest),
        "testnet" | "testnet3" => Ok(Network::Testnet),
        "signet" => Ok(Network::Signet),
        "bitcoin" | "mainnet" => Ok(Network::Bitcoin),
        other => Err(Error::Config(format!(
            "unknown network `{other}`; expected regtest, testnet or signet"
        ))),
    }
}

fn default_rpc_url(network: Network) -> String {
    let port = match network {
        Network::Bitcoin => 8332,
        Network::Testnet => 18332,
        Network::Signet => 38332,
        _ => 18443,
    };
    format!("http://127.0.0.1:{port}")
}

/// Explicit user/password wins; otherwise fall back to the cookie file that
/// `bitcoind` writes into its data directory.
fn resolve_auth(network: Network) -> Result<RpcAuth> {
    let user = std::env::var("RFB_RPC_USER").ok().filter(|s| !s.is_empty());
    let pass = std::env::var("RFB_RPC_PASSWORD")
        .ok()
        .filter(|s| !s.is_empty());

    match (user, pass) {
        (Some(user), Some(pass)) => return Ok(RpcAuth::UserPass(user, pass)),
        (Some(_), None) => {
            return Err(Error::Config(
                "RFB_RPC_USER is set but RFB_RPC_PASSWORD is not".into(),
            ));
        }
        (None, Some(_)) => {
            return Err(Error::Config(
                "RFB_RPC_PASSWORD is set but RFB_RPC_USER is not".into(),
            ));
        }
        (None, None) => {}
    }

    if let Ok(path) = std::env::var("RFB_RPC_COOKIE") {
        let path = PathBuf::from(path);
        if !path.exists() {
            return Err(Error::Config(format!(
                "RFB_RPC_COOKIE points at {}, which does not exist",
                path.display()
            )));
        }
        return Ok(RpcAuth::Cookie(path));
    }

    let path = default_cookie_path(network);
    if path.exists() {
        Ok(RpcAuth::Cookie(path))
    } else {
        Err(Error::Config(format!(
            "no RPC credentials: set RFB_RPC_USER and RFB_RPC_PASSWORD, or RFB_RPC_COOKIE, \
             or start bitcoind so that {} exists",
            path.display()
        )))
    }
}

/// Where `bitcoind` puts its cookie by default on this platform.
fn default_cookie_path(network: Network) -> PathBuf {
    let base = if cfg!(target_os = "macos") {
        dirs_home().join("Library/Application Support/Bitcoin")
    } else if cfg!(target_os = "windows") {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs_home())
            .join("Bitcoin")
    } else {
        dirs_home().join(".bitcoin")
    };

    let sub: &Path = match network {
        Network::Bitcoin => Path::new(""),
        Network::Testnet => Path::new("testnet3"),
        Network::Signet => Path::new("signet"),
        _ => Path::new("regtest"),
    };

    base.join(sub).join(".cookie")
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_supported_networks() {
        assert_eq!(parse_network("regtest").unwrap(), Network::Regtest);
        assert_eq!(parse_network("  TestNet ").unwrap(), Network::Testnet);
        assert_eq!(parse_network("testnet3").unwrap(), Network::Testnet);
        assert_eq!(parse_network("signet").unwrap(), Network::Signet);
        assert!(parse_network("liquid").is_err());
    }

    #[test]
    fn mainnet_parses_but_is_refused_higher_up() {
        // `parse_network` stays honest about what the string means; refusing it is
        // `Config::from_env`'s job, so the error can say why.
        assert_eq!(parse_network("mainnet").unwrap(), Network::Bitcoin);
    }

    #[test]
    fn default_ports_match_bitcoin_core() {
        assert_eq!(default_rpc_url(Network::Regtest), "http://127.0.0.1:18443");
        assert_eq!(default_rpc_url(Network::Testnet), "http://127.0.0.1:18332");
        assert_eq!(default_rpc_url(Network::Signet), "http://127.0.0.1:38332");
    }

    #[test]
    fn cookie_paths_are_network_scoped() {
        let regtest = default_cookie_path(Network::Regtest);
        let signet = default_cookie_path(Network::Signet);
        assert!(regtest.ends_with("regtest/.cookie"));
        assert!(signet.ends_with("signet/.cookie"));
        assert_ne!(regtest, signet);
    }

    #[test]
    fn rpc_auth_hides_the_password() {
        let auth = RpcAuth::UserPass("rfb".into(), "hunter2".into());
        let rendered = format!("{auth:?}");
        assert!(rendered.contains("rfb"));
        assert!(!rendered.contains("hunter2"));
    }
}
