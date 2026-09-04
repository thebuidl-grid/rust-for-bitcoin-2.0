mod cli;
mod commands;
mod config;
mod node;
mod wallet;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Load .env — log whether it was found so we can debug missing-env issues.
    match dotenv::dotenv() {
        Ok(path) => log::debug!("Loaded .env from {:?}", path),
        Err(e) => log::warn!("Could not load .env: {e}"),
    }

    let cli = Cli::parse();

    match cli.command {
        Command::Address => commands::get_address()?,
        Command::Balance => commands::get_balance()?,
        Command::Sync => commands::sync()?,
        Command::Send { to, amount_sats } => commands::send(&to, amount_sats)?,
    }

    Ok(())
}
