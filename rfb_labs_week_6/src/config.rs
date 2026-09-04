use std::env;
use std::path::PathBuf;

use bdk_wallet::bitcoin::Network;

/// Everything the wallet needs to know, read from the environment (`.env` or real env vars).
///
/// Nothing here is hardcoded — the mnemonic in particular is read at runtime and never
/// committed. See `init` in main.rs for how it gets generated the first time.
pub struct Config {
    pub network: Network,
    pub db_path: PathBuf,
    pub rpc_url: String,
    pub rpc_cookie: Option<PathBuf>,
    pub rpc_user: Option<String>,
    pub rpc_pass: Option<String>,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        // Ignore "not found" — real env vars can carry all of this instead.
        let _ = dotenvy::dotenv();

        let network = match env::var("BITCOIN_NETWORK").as_deref() {
            Ok("regtest") | Err(_) => Network::Regtest,
            Ok("testnet") => Network::Testnet,
            Ok("signet") => Network::Signet,
            Ok(other) => anyhow::bail!("unsupported BITCOIN_NETWORK: {other}"),
        };

        let db_path = env::var("BDK_DB_PATH")
            .unwrap_or_else(|_| "wallet.sqlite".to_owned())
            .into();

        let rpc_url = env::var("RPC_URL").unwrap_or_else(|_| "127.0.0.1:18443".to_owned());
        let rpc_cookie = env::var("RPC_COOKIE").ok().map(PathBuf::from);
        let rpc_user = env::var("RPC_USER").ok();
        let rpc_pass = env::var("RPC_PASS").ok();

        Ok(Self {
            network,
            db_path,
            rpc_url,
            rpc_cookie,
            rpc_user,
            rpc_pass,
        })
    }

    pub fn mnemonic_phrase() -> Option<String> {
        env::var("MNEMONIC").ok()
    }
}
