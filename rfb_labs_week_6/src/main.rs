//! A descriptor wallet for Bitcoin regtest.
//!
//! Three libraries, three jobs:
//!
//! - `rust-bitcoin` owns the primitives: BIP32 derivation, addresses, script
//!   types, consensus (de)serialisation and sighash computation.
//! - `bitcoincore-rpc` is the only thing that talks to the network. The node is
//!   used as a chain source and a relay, not as a wallet.
//! - `bdk_wallet` sits between them and keeps the state a wallet needs: which
//!   scripts belong to us, which outputs are unspent, what the balance is, and
//!   how to turn a spend request into a signed transaction.

mod cli;
mod commands;
mod config;
mod error;
mod keys;
mod node;
mod raw;
mod store;
mod sync;
mod tx;
mod ui;
mod wallet;

use clap::Parser;
use tracing_subscriber::EnvFilter;

use cli::{Cli, Command};
use config::Config;
use error::Result;

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    init_logging(cli.verbose);

    match run(&cli) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("\nerror: {err}");

            // Print the chain of causes, skipping any that the message above
            // already spells out. Library errors often embed their own source,
            // and repeating it verbatim adds noise rather than detail.
            let mut previous = err.to_string();
            let mut source = std::error::Error::source(&err);
            while let Some(cause) = source {
                let message = cause.to_string();
                if !previous.contains(&message) {
                    eprintln!("  caused by: {message}");
                }
                previous = message;
                source = cause.source();
            }

            std::process::ExitCode::FAILURE
        }
    }
}

fn run(cli: &Cli) -> Result<()> {
    // `new-mnemonic` must work before any configuration exists, so it is handled
    // before the config is loaded.
    if matches!(cli.command, Command::NewMnemonic) {
        return commands::new_mnemonic();
    }

    let cfg = Config::from_env()?;
    tracing::debug!(?cfg.network, %cfg.rpc_url, "loaded configuration");

    match &cli.command {
        Command::Init(args) => commands::init(&cfg, args),
        Command::Info => commands::info(&cfg),
        Command::Address(args) => commands::address(&cfg, args),
        Command::Balance => commands::balance(&cfg),
        Command::Utxos(args) => commands::utxos(&cfg, args),
        Command::Txs => commands::txs(&cfg),
        Command::Sync(args) => commands::sync_cmd(&cfg, args),
        Command::Send(args) => commands::send(&cfg, args),
        Command::Node => commands::node_info(&cfg),
        Command::Mine(args) => commands::mine(&cfg, args),
        Command::Verify(args) => commands::verify(&cfg, args),
        Command::Compare => commands::compare(&cfg),
        Command::NewMnemonic => commands::new_mnemonic(),
    }
}

fn init_logging(verbose: bool) {
    let default = if verbose {
        "rfb_labs_week_6=debug,bdk_wallet=debug,bdk_bitcoind_rpc=debug"
    } else {
        "warn"
    };
    let filter = EnvFilter::try_from_env("RFB_LOG").unwrap_or_else(|_| EnvFilter::new(default));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .without_time()
        .with_writer(std::io::stderr)
        .init();
}
