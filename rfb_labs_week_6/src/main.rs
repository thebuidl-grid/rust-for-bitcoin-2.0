//! A minimal regtest/testnet Bitcoin wallet CLI built on `bdk_wallet` and
//! `bitcoincore-rpc`. See `Readme.md` for setup and usage.

mod commands;
mod config;
mod descriptors;
mod mnemonic;
mod sync;

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use config::{Cli, Command};

fn main() -> Result<()> {
    let env_path = PathBuf::from(".env");
    dotenvy::from_path(&env_path).ok();

    let mut cli = Cli::parse();

    match cli.command {
        Command::Init => commands::init(&mut cli.config, &env_path),
        Command::Address => commands::address(&cli.config),
        Command::ChangeAddress => commands::change_address(&cli.config),
        Command::Sync => commands::sync(&cli.config),
        Command::Balance => commands::balance(&cli.config),
        Command::Utxos => commands::utxos(&cli.config),
        Command::Send {
            address,
            amount_sats,
        } => commands::send(&cli.config, &address, amount_sats),
    }
}
