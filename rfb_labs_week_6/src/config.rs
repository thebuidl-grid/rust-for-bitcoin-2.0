//! Runtime configuration, loaded from the environment (and `.env` if present).
//!
//! Nothing secret is compiled in. The mnemonic lives only in `.env`, which is
//! gitignored, per the assignment's "do not hardcode private keys or seed phrases"
//! constraint.
//!
//! Everything here is validated once at startup so the rest of the crate can work
//! with typed values (`Network`, `PathBuf`, `DescriptorKind`) instead of re-parsing
//! strings at each use.

use std::path::PathBuf;
use std::str::FromStr;

use bdk_wallet::bitcoin::{Network, NetworkKind};

use crate::error::{Result, WalletError};

// Environment keys, named once so a typo is a compile error rather than a silent
// "missing setting" at runtime.
const ENV_NETWORK: &str = "BITCOIN_NETWORK";
const ENV_MNEMONIC: &str = "WALLET_MNEMONIC";
const ENV_PASSPHRASE: &str = "WALLET_PASSPHRASE";
const ENV_DB: &str = "WALLET_DB";
const ENV_KIND: &str = "DESCRIPTOR_KIND";
const ENV_RPC_URL: &str = "RPC_URL";
const ENV_RPC_USER: &str = "RPC_USER";
const ENV_RPC_PASSWORD: &str = "RPC_PASSWORD";

/// Which script type the wallet's descriptors use.
///
/// Both derive from the same seed; they differ in BIP-43 purpose and in the
/// descriptor fragment BDK expands. Keeping this an enum is what makes the
/// "compare `wpkh` vs `tr`" stretch goal a one-line config change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DescriptorKind {
    /// BIP84 — native segwit v0, `wpkh(...)`, addresses start `bcrt1q...`
    #[default]
    Wpkh,
    /// BIP86 — Taproot, `tr(...)`, addresses start `bcrt1p...`
    Tr,
}

impl DescriptorKind {
    /// BIP-43 purpose field: the first hardened index of the derivation path.
    pub fn purpose(self) -> u32 {
        match self {
            Self::Wpkh => 84,
            Self::Tr => 86,
        }
    }
}

impl std::fmt::Display for DescriptorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wpkh => write!(f, "wpkh"),
            Self::Tr => write!(f, "tr"),
        }
    }
}

impl FromStr for DescriptorKind {
    type Err = WalletError;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "wpkh" | "bip84" | "segwit" => Ok(Self::Wpkh),
            "tr" | "bip86" | "taproot" => Ok(Self::Tr),
            other => Err(WalletError::InvalidEnv {
                key: ENV_KIND,
                value: other.to_string(),
                reason: "expected one of: wpkh, tr",
            }),
        }
    }
}

/// How to reach the Bitcoin Core node.
#[derive(Debug, Clone)]
pub struct RpcConfig {
    pub url: String,
    pub user: String,
    pub password: String,
}

/// Validated runtime configuration.
#[derive(Debug, Clone)]
pub struct Config {
    pub network: Network,
    /// `None` until `init` has generated one. Commands that need it call
    /// [`Config::mnemonic`], which turns absence into a helpful error.
    pub mnemonic: Option<String>,
    /// Optional BIP39 passphrase (the "25th word").
    pub passphrase: Option<String>,
    pub db_path: PathBuf,
    pub descriptor_kind: DescriptorKind,
    pub rpc: RpcConfig,
}

impl Config {
    /// Read `.env` (if present) and the process environment into a validated
    /// `Config`. A missing `.env` is fine — the environment alone may supply
    /// everything, which is how CI and the tests run.
    pub fn load() -> Result<Self> {
        match dotenvy::dotenv() {
            Ok(_) => {}
            Err(e) if e.not_found() => {}
            Err(dotenvy::Error::Io(e)) => return Err(WalletError::Io(e)),
            Err(e) => {
                return Err(WalletError::InvalidEnv {
                    key: ".env",
                    value: e.to_string(),
                    reason: "could not be parsed",
                });
            }
        }

        let network = parse_network(&env_or(ENV_NETWORK, "regtest"))?;
        let descriptor_kind = env_or(ENV_KIND, "wpkh").parse()?;

        Ok(Self {
            network,
            mnemonic: env_opt(ENV_MNEMONIC),
            passphrase: env_opt(ENV_PASSPHRASE),
            db_path: PathBuf::from(env_or(ENV_DB, "./wallet.sqlite")),
            descriptor_kind,
            rpc: RpcConfig {
                // Defaults are Polar's published regtest credentials. They are not
                // secrets — Polar ships the same -rpcauth hash in every network — so
                // defaulting them keeps first-run friction low. Override in .env for
                // any other node.
                url: env_or(ENV_RPC_URL, "http://127.0.0.1:18443"),
                user: env_or(ENV_RPC_USER, "polaruser"),
                password: env_or(ENV_RPC_PASSWORD, "polarpass"),
            },
        })
    }

    /// The mnemonic, or a message telling the user how to get one.
    pub fn mnemonic(&self) -> Result<&str> {
        self.mnemonic
            .as_deref()
            .ok_or(WalletError::MissingEnv(ENV_MNEMONIC))
    }

    /// `NetworkKind` is what descriptor templates take. It collapses every test
    /// network to `Test`, which is why regtest correctly derives at coin type
    /// `1'` rather than mainnet's `0'`.
    pub fn network_kind(&self) -> NetworkKind {
        NetworkKind::from(self.network)
    }
}

/// Parse a network name and enforce the assignment's "testnet or regtest only"
/// constraint.
///
/// Mainnet parses successfully as far as `rust-bitcoin` is concerned, so refusing
/// it is a policy decision we make here rather than something the library does for
/// us. Unknown names and mainnet produce the same clear error.
fn parse_network(value: &str) -> Result<Network> {
    let network = Network::from_str(value.trim())
        .map_err(|_| WalletError::UnsupportedNetwork(value.to_string()))?;

    match network {
        Network::Bitcoin => Err(WalletError::UnsupportedNetwork(value.to_string())),
        other => Ok(other),
    }
}

/// Restrict a file to owner read/write only.
///
/// Used for `.env` (holds the seed) and the wallet database (holds the account
/// xpub plus your whole address and transaction history). Neither should be
/// world-readable merely because the umask was permissive.
pub(crate) fn restrict_permissions(path: &std::path::Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = std::fs::metadata(path)?.permissions();
        if perms.mode() & 0o777 != 0o600 {
            perms.set_mode(0o600);
            std::fs::set_permissions(path, perms)?;
        }
    }
    #[cfg(not(unix))]
    let _ = path;

    Ok(())
}

/// Read a key, falling back to a default. Whitespace-only values count as unset.
fn env_or(key: &'static str, default: &str) -> String {
    env_opt(key).unwrap_or_else(|| default.to_string())
}

/// Read a key as `Option`, treating empty/whitespace as absent.
///
/// This matters because `init` writes a scaffold `.env` containing
/// `WALLET_MNEMONIC=` with no value. Without this, that empty string would be
/// handed to the BIP39 parser and fail with a confusing checksum error instead of
/// "run init first".
fn env_opt(key: &'static str) -> Option<String> {
    match std::env::var(key) {
        Ok(v) if !v.trim().is_empty() => Some(v.trim().to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_mainnet() {
        let err = parse_network("bitcoin").unwrap_err();
        assert!(matches!(err, WalletError::UnsupportedNetwork(_)));
    }

    #[test]
    fn rejects_unknown_network() {
        assert!(parse_network("mainnet").is_err());
        assert!(parse_network("").is_err());
    }

    #[test]
    fn accepts_test_networks() {
        assert_eq!(parse_network("regtest").unwrap(), Network::Regtest);
        assert_eq!(parse_network("signet").unwrap(), Network::Signet);
        assert_eq!(parse_network("testnet").unwrap(), Network::Testnet);
        assert_eq!(parse_network("testnet4").unwrap(), Network::Testnet4);
    }

    #[test]
    fn regtest_derives_at_testnet_coin_type() {
        // NetworkKind::Test is what drives coin type 1' in the BIP84/86 templates.
        assert_eq!(NetworkKind::from(Network::Regtest), NetworkKind::Test);
    }

    #[test]
    fn descriptor_kind_parsing() {
        assert_eq!("wpkh".parse::<DescriptorKind>().unwrap(), DescriptorKind::Wpkh);
        assert_eq!("Taproot".parse::<DescriptorKind>().unwrap(), DescriptorKind::Tr);
        assert!("p2pkh".parse::<DescriptorKind>().is_err());
        assert_eq!(DescriptorKind::Wpkh.purpose(), 84);
        assert_eq!(DescriptorKind::Tr.purpose(), 86);
    }
}
