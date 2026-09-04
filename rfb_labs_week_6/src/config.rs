//! Loads all wallet configuration from the environment / a local `.env` file.
//!
//! Nothing sensitive is ever hardcoded in source: the recovery mnemonic, network, and node
//! credentials all come from the environment. On first run (no `WALLET_MNEMONIC` set), a fresh
//! test mnemonic is generated and appended to `.env` so the same wallet is reused on every
//! subsequent run. `.env` is git-ignored.

use std::env;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use bdk_wallet::bitcoin::Network;
use bdk_wallet::keys::bip39::{Language, Mnemonic, WordCount};
use bdk_wallet::keys::{GeneratableKey, GeneratedKey};
use bdk_wallet::miniscript::Segwitv0;

const ENV_FILE: &str = ".env";

pub struct Config {
    pub network: Network,
    pub mnemonic: Mnemonic,
    pub passphrase: String,
    pub rpc_url: String,
    pub rpc_user: String,
    pub rpc_pass: String,
    pub db_path: PathBuf,
}

impl Config {
    pub fn load() -> Result<Self> {
        // Ignored if `.env` doesn't exist yet -- env vars set directly still work.
        dotenvy::dotenv().ok();

        let network: Network = env::var("BITCOIN_NETWORK")
            .unwrap_or_else(|_| "regtest".to_owned())
            .parse()
            .context("invalid BITCOIN_NETWORK (expected bitcoin/testnet/signet/regtest)")?;

        if network == Network::Bitcoin {
            return Err(anyhow!(
                "BITCOIN_NETWORK=bitcoin is refused by this assignment: regtest/testnet/signet only"
            ));
        }

        let mnemonic = load_or_generate_mnemonic()?;
        let passphrase = env::var("WALLET_PASSPHRASE").unwrap_or_default();

        let rpc_url = env::var("RPC_URL").unwrap_or_else(|_| "127.0.0.1:18443".to_owned());
        let rpc_user = env::var("RPC_USER").unwrap_or_else(|_| "polaruser".to_owned());
        let rpc_pass = env::var("RPC_PASS").unwrap_or_else(|_| "polarpass".to_owned());
        let db_path = env::var("WALLET_DB_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("wallet.sqlite"));

        Ok(Self {
            network,
            mnemonic,
            passphrase,
            rpc_url,
            rpc_user,
            rpc_pass,
            db_path,
        })
    }
}

fn load_or_generate_mnemonic() -> Result<Mnemonic> {
    if let Ok(phrase) = env::var("WALLET_MNEMONIC") {
        return Mnemonic::parse_in(Language::English, phrase.trim())
            .context("WALLET_MNEMONIC is not a valid BIP39 mnemonic");
    }

    let generated: GeneratedKey<Mnemonic, Segwitv0> =
        Mnemonic::generate((WordCount::Words12, Language::English))
            .map_err(|_| anyhow!("failed to generate a fresh BIP39 mnemonic"))?;
    let mnemonic = generated.into_key();

    eprintln!(
        "No WALLET_MNEMONIC set -- generated a new disposable test mnemonic and saved it to \
         {ENV_FILE} for reuse on future runs. This is test data for regtest/testnet only; \
         never fund it with real bitcoin."
    );

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(ENV_FILE)
        .with_context(|| format!("failed to open {ENV_FILE} to save the generated mnemonic"))?;
    writeln!(file, "WALLET_MNEMONIC=\"{mnemonic}\"")
        .with_context(|| format!("failed to write the generated mnemonic to {ENV_FILE}"))?;

    Ok(mnemonic)
}
