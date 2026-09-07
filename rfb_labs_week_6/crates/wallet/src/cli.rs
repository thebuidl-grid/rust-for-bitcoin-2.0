use std::path::PathBuf;

use bitcoin::Network;
use clap::{Parser, Subcommand, ValueEnum};
use directories::ProjectDirs;

use crate::{
    config::{RpcConfig, WalletConfig},
    error::{WalletError, WalletResult},
};

#[derive(Debug, Parser)]
#[command(
    name = "rfb-wallet",
    version,
    about = "A descriptor-based Bitcoin regtest wallet"
)]
pub struct Cli {
    /// Bitcoin network to use.
    #[arg(long, env = "RFB_NETWORK", value_enum, default_value_t = NetworkArg::Regtest)]
    pub network: NetworkArg,

    /// Directory used for the wallet database and local state.
    #[arg(long, env = "RFB_DATA_DIR")]
    pub data_dir: Option<PathBuf>,

    /// Bitcoin Core JSON-RPC endpoint.
    #[arg(long, env = "RFB_RPC_URL", default_value = "http://127.0.0.1:18443")]
    pub rpc_url: String,

    /// Bitcoin Core RPC username. Prefer the RFB_RPC_USER environment variable.
    #[arg(long, env = "RFB_RPC_USER")]
    pub rpc_user: Option<String>,

    /// Bitcoin Core RPC password. Prefer the RFB_RPC_PASSWORD environment variable.
    #[arg(long, env = "RFB_RPC_PASSWORD", hide_env_values = true)]
    pub rpc_password: Option<String>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum NetworkArg {
    Regtest,
    Testnet,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a new descriptor wallet and persist it locally.
    Init,
    /// Derive the next receiving or change address.
    Address {
        /// Derive from the internal/change keychain.
        #[arg(long)]
        change: bool,
    },
    /// Synchronize wallet state with Bitcoin Core.
    Sync,
    /// Display confirmed, pending, immature, and total balance.
    Balance,
    /// List wallet-controlled unspent transaction outputs.
    Utxos,
    /// Construct, sign, and broadcast a transaction.
    Send {
        /// Network-valid destination address.
        #[arg(long)]
        to: String,
        /// Amount to send in satoshis.
        #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
        amount: u64,
        /// Fee rate in satoshis per virtual byte.
        #[arg(long, default_value_t = 2.0)]
        fee_rate: f32,
    },
}

impl Cli {
    pub fn wallet_config(&self) -> WalletResult<WalletConfig> {
        let data_dir = match &self.data_dir {
            Some(path) => path.clone(),
            None => ProjectDirs::from("dev", "mufasa-org", "muf-wallet")
                .map(|dirs| dirs.data_local_dir().to_path_buf())
                .ok_or_else(|| {
                    WalletError::Config("could not determine a data directory".into())
                })?,
        };

        let network = match self.network {
            NetworkArg::Regtest => Network::Regtest,
            NetworkArg::Testnet => Network::Testnet,
        };
        let rpc = RpcConfig::new(
            self.rpc_url.clone(),
            self.rpc_user.clone(),
            self.rpc_password.clone(),
        );

        WalletConfig::new(network, data_dir, rpc)
    }
}

impl Command {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Init => "init",
            Self::Address { .. } => "address",
            Self::Sync => "sync",
            Self::Balance => "balance",
            Self::Utxos => "utxos",
            Self::Send { .. } => "send",
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Cli, Command, NetworkArg};

    #[test]
    fn defaults_to_regtest() {
        let cli = Cli::try_parse_from(["rfb-wallet", "balance"]).unwrap();

        assert_eq!(cli.network, NetworkArg::Regtest);
        assert!(matches!(cli.command, Command::Balance));
    }

    #[test]
    fn rejects_zero_value_send() {
        let result = Cli::try_parse_from([
            "rfb-wallet",
            "send",
            "--to",
            "bcrt1qexample",
            "--amount",
            "0",
        ]);

        assert!(result.is_err());
    }
}
