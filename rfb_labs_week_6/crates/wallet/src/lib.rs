//! Descriptor-based Bitcoin wallet application library.

pub mod cli;
pub mod config;
pub mod core;
pub mod error;
pub mod logging;
pub mod node;
pub mod persistence;
pub mod types;

pub use core::WalletService;

use clap::Parser;
use cli::{Cli, Command};
use config::WalletConfig;
use error::{WalletError, WalletResult};

/// Parses process arguments and runs the selected wallet command.
pub fn run() -> WalletResult<()> {
    load_environment()?;
    logging::init();

    let cli = Cli::parse();
    let config = cli.wallet_config()?;
    Application::new(config).execute(cli.command)
}

fn load_environment() -> WalletResult<()> {
    match dotenvy::dotenv() {
        Ok(_) => Ok(()),
        Err(dotenvy::Error::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

/// Thin orchestration layer between presentation and wallet services.
struct Application {
    config: WalletConfig,
}

impl Application {
    fn new(config: WalletConfig) -> Self {
        Self { config }
    }

    fn execute(self, command: Command) -> WalletResult<()> {
        tracing::debug!(network = %self.config.network, command = ?command, "executing command");

        // Command arms will call WalletService as each use case is implemented.
        // Terminal formatting remains here rather than in domain modules.
        Err(WalletError::NotImplemented(command.name()))
    }
}
