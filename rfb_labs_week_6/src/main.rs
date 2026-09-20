mod cli;
mod config;
mod node;
mod seed;
mod send;
mod wallet;

use anyhow::{Context, Result};
use bdk_wallet::KeychainKind;
use bdk_wallet::bitcoin::{Address, Amount};
use clap::Parser;

use cli::{Cli, Command};
use config::Config;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::from_env()?;

    let client = node::connect(&config)?;
    node::check_connection(&client)?;

    let mnemonic = seed::resolve_mnemonic()?;
    let (mut wallet, mut db) = wallet::open_or_create(&mnemonic, config.network)?;

    match cli.command {
        Command::Sync => {
            let before = wallet.latest_checkpoint().height();
            wallet::sync(&mut wallet, &mut db, &config)?;
            let after = wallet.latest_checkpoint().height();
            println!("Wallet checkpoint: {before} -> {after}");
            print_balance(&wallet);
        }
        Command::Balance => {
            wallet::sync(&mut wallet, &mut db, &config)?;
            print_balance(&wallet);
        }
        Command::Address { internal } => {
            let keychain = if internal {
                KeychainKind::Internal
            } else {
                KeychainKind::External
            };
            let address = wallet.reveal_next_address(keychain);
            wallet.persist(&mut db)?;
            println!("{}", address.address);
        }
        Command::Send {
            address,
            amount_btc,
        } => {
            wallet::sync(&mut wallet, &mut db, &config)?;

            let to_address: Address<_> = address
                .parse::<Address<_>>()
                .context("invalid address")?
                .require_network(config.network)
                .context("address is for the wrong network")?;
            let amount = Amount::from_btc(amount_btc).context("invalid amount")?;

            let txid = send::send(&mut wallet, &mut db, &config, &to_address, amount)?;
            println!("Broadcast! txid: {txid}");
        }
    }

    Ok(())
}

fn print_balance(wallet: &bdk_wallet::PersistedWallet<bdk_wallet::rusqlite::Connection>) {
    let balance = wallet.balance();
    println!("Balance breakdown:");
    println!("  confirmed:          {}", balance.confirmed);
    println!("  trusted pending:    {}", balance.trusted_pending);
    println!("  untrusted pending:  {}", balance.untrusted_pending);
    println!("  immature:           {}", balance.immature);
    println!("  total:              {}", balance.total());
}
