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
        tracing::debug!(network = %self.config.network, command = command.name(), "executing command");

        match command {
            Command::Init { mnemonic } => {
                let (_service, summary) =
                    WalletService::initialize(&self.config, mnemonic.as_deref())?;

                println!();
                println!("Wallet initialized successfully.");
                println!();
                println!("  Network:  {}", summary.network);
                println!("  Database: {}", summary.database_path.display());
                println!();
                println!("Receiving descriptor:");
                println!("  {}", summary.external_descriptor);
                println!();
                println!("Change descriptor:");
                println!("  {}", summary.internal_descriptor);

                if let Some(recovery_phrase) = summary.recovery_phrase {
                    println!();
                    println!("Recovery phrase:");
                    println!("  {recovery_phrase}");
                    println!();
                    println!(
                        "Back it up now. To restore signing access, set MUF_MNEMONIC in your private .env file."
                    );
                }

                println!();

                Ok(())
            }
            other => Err(WalletError::NotImplemented(other.name())),
        }
    }
}
