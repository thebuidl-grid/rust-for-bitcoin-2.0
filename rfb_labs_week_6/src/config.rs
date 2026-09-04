use std::fs;
use std::path::{Path, PathBuf};

use bdk_wallet::bitcoin::Network;

use crate::error::WalletError;
use crate::keys::{self, DescriptorKind};

const ENV_FILE: &str = ".env";
const ENV_EXAMPLE: &str = ".env.example";

const DEFAULT_ENV: &str = "\
NETWORK=regtest
RPC_URL=127.0.0.1:18443
# RPC auth: set a cookie file path, OR set RPC_USER and RPC_PASS
RPC_COOKIE=
# RPC_USER=
# RPC_PASS=
MNEMONIC=
# PASSPHRASE=
DESCRIPTOR_KIND=wpkh
DB_PATH=wallet.sqlite
START_HEIGHT=0
";

// === Types

/// How to authenticate against Bitcoin Core's RPC interface.
#[derive(Debug, Clone)]
pub enum RpcAuth {
    /// Path to the `.cookie` file Core writes into its data directory.
    Cookie(PathBuf),
    /// A static `rpcuser` / `rpcpassword` pair.
    UserPass(String, String),
}

/// Everything the wallet needs to run, resolved from environment / `.env`.
#[derive(Debug, Clone)]
pub struct Config {
    pub network: Network,
    pub rpc_url: String,
    pub rpc_auth: RpcAuth,
    pub mnemonic: String,
    pub passphrase: Option<String>,
    pub descriptor_kind: DescriptorKind,
    pub db_path: PathBuf,
    pub start_height: u32,
}

impl Config {
    /// Load `.env` (if present) and build a `Config` from the environment.
    pub fn load() -> Result<Self, WalletError> {
        dotenvy::dotenv().ok();
        return Self::from_env();
    }

    fn from_env() -> Result<Self, WalletError> {
        let network = match env_opt("NETWORK") {
            Some(v) => v
                .parse::<Network>()
                .map_err(|_| WalletError::Config(format!("invalid NETWORK '{v}'")))?,
            None => Network::Regtest,
        };
        if network == Network::Bitcoin {
            return Err(WalletError::Config(
                "mainnet is not allowed; use regtest, testnet or signet".to_string(),
            ));
        }

        let rpc_url = env_opt("RPC_URL").unwrap_or_else(|| "127.0.0.1:18443".to_string());

        let rpc_auth = match (
            env_opt("RPC_COOKIE"),
            env_opt("RPC_USER"),
            env_opt("RPC_PASS"),
        ) {
            (Some(path), _, _) => RpcAuth::Cookie(PathBuf::from(path)),
            (None, Some(user), Some(pass)) => RpcAuth::UserPass(user, pass),
            _ => {
                return Err(WalletError::Config(
                    "set RPC_COOKIE to a cookie file path, or set both RPC_USER and RPC_PASS"
                        .to_string(),
                ));
            }
        };

        let mnemonic = env_opt("MNEMONIC").ok_or_else(|| {
            WalletError::Config(
                "MNEMONIC is not set; run `rfbwallet init` to generate one".to_string(),
            )
        })?;

        let passphrase = env_opt("PASSPHRASE");

        let descriptor_kind = match env_opt("DESCRIPTOR_KIND") {
            Some(v) => v.parse()?,
            None => DescriptorKind::Wpkh,
        };

        let db_path =
            PathBuf::from(env_opt("DB_PATH").unwrap_or_else(|| "wallet.sqlite".to_string()));

        let start_height = match env_opt("START_HEIGHT") {
            Some(v) => v
                .parse()
                .map_err(|_| WalletError::Config(format!("invalid START_HEIGHT '{v}'")))?,
            None => 0,
        };

        return Ok(Config {
            network,
            rpc_url,
            rpc_auth,
            mnemonic,
            passphrase,
            descriptor_kind,
            db_path,
            start_height,
        });
    }
}

fn env_opt(key: &str) -> Option<String> {
    return std::env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());
}

// === init bootstrap

/// Ensure `.env` exists and contains a mnemonic before the first wallet is
/// created. Called only by `rfbwallet init`.
///
/// - If `.env` is missing it is seeded from `.env.example` (or a builtin default).
/// - If `MNEMONIC` is empty a fresh one is generated and written back.
/// - `--taproot` forces `DESCRIPTOR_KIND=tr`.
pub fn bootstrap_env(taproot: bool) -> Result<(PathBuf, bool), WalletError> {
    let env_path = Path::new(ENV_FILE);

    if !env_path.exists() {
        if Path::new(ENV_EXAMPLE).exists() {
            fs::copy(ENV_EXAMPLE, ENV_FILE)?;
        } else {
            fs::write(ENV_FILE, DEFAULT_ENV)?;
        }
    }

    let mut content = fs::read_to_string(env_path)?;
    let mut generated = false;

    if current_value(&content, "MNEMONIC")
        .filter(|v| !v.is_empty())
        .is_none()
    {
        let mnemonic = keys::generate_mnemonic()?;
        content = set_kv(&content, "MNEMONIC", &format!("\"{mnemonic}\""));
        generated = true;
    }

    if taproot {
        content = set_kv(&content, "DESCRIPTOR_KIND", "tr");
    } else if current_value(&content, "DESCRIPTOR_KIND").is_none() {
        content = set_kv(&content, "DESCRIPTOR_KIND", "wpkh");
    }

    fs::write(env_path, &content)?;
    dotenvy::from_path_override(env_path).ok();

    return Ok((env_path.to_path_buf(), generated));
}

fn current_value(content: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}=");
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            return Some(rest.trim().trim_matches('"').to_string());
        }
    }
    return None;
}

fn set_kv(content: &str, key: &str, value: &str) -> String {
    let prefix = format!("{key}=");
    let mut replaced = false;
    let mut out: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('#') && trimmed.starts_with(&prefix) {
            out.push(format!("{key}={value}"));
            replaced = true;
        } else {
            out.push(line.to_string());
        }
    }
    if !replaced {
        out.push(format!("{key}={value}"));
    }

    return out.join("\n") + "\n";
}
