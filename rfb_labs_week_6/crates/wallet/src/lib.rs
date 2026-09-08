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

use bitcoin::{Amount, FeeRate};
use clap::Parser;
use cli::{Cli, Command};
use config::WalletConfig;
use error::{WalletError, WalletResult};
use node::{BitcoinCoreNode, NodeBackend};
use types::Keychain;

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
            Command::Address { change } => {
                let mut service = WalletService::load(&self.config, None)?;
                let keychain = if change {
                    Keychain::Internal
                } else {
                    Keychain::External
                };
                let derived = service.next_address(keychain)?;
                let purpose = if change { "Change" } else { "Receiving" };

                println!("{purpose} address #{}:", derived.derivation_index);
                println!("  {}", derived.address);

                Ok(())
            }
            Command::NodeHealth => {
                let node = BitcoinCoreNode::connect(&self.config.rpc, self.config.network)?;
                let height = node.tip_height()?;

                println!("Bitcoin Core connection is healthy.");
                println!("  Network: {}", self.config.network);
                println!("  Height:  {height}");
                println!("  RPC:     {}", self.config.rpc.url);

                Ok(())
            }
            Command::Sync => {
                let mut service = WalletService::load(&self.config, None)?;
                let node = BitcoinCoreNode::connect(&self.config.rpc, self.config.network)?;
                let summary = service.sync(&node)?;

                println!("Wallet synchronized successfully.");
                println!(
                    "  Tip:             {}:{}",
                    summary.tip_height, summary.tip_hash
                );
                println!("  Blocks applied:  {}", summary.blocks_applied);
                println!("  Mempool txs:     {}", summary.mempool_transactions);
                println!("  Evicted txs:     {}", summary.evicted_transactions);

                Ok(())
            }
            Command::Balance { offline } => {
                let mut service = WalletService::load(&self.config, None)?;
                synchronize_unless_offline(&self.config, &mut service, offline)?;
                let balance = service.balance()?;

                println!("Wallet balance:");
                println!(
                    "  Confirmed:            {} sats",
                    balance.confirmed.to_sat()
                );
                println!(
                    "  Pending (trusted):     {} sats",
                    balance.trusted_pending.to_sat()
                );
                println!(
                    "  Pending (untrusted):   {} sats",
                    balance.untrusted_pending.to_sat()
                );
                println!("  Pending (total):       {} sats", balance.pending.to_sat());
                println!("  Immature:             {} sats", balance.immature.to_sat());
                println!(
                    "  Spendable:            {} sats",
                    balance.spendable.to_sat()
                );
                println!("  Total:                {} sats", balance.total.to_sat());

                Ok(())
            }
            Command::Utxos { offline } => {
                let mut service = WalletService::load(&self.config, None)?;
                synchronize_unless_offline(&self.config, &mut service, offline)?;
                let utxos = service.list_utxos()?;

                if utxos.is_empty() {
                    println!("No wallet UTXOs found. Run `sync` after funding a wallet address.");
                    return Ok(());
                }

                println!("Wallet UTXOs ({}):", utxos.len());
                for utxo in utxos {
                    let keychain = match utxo.keychain {
                        Keychain::External => "receiving",
                        Keychain::Internal => "change",
                    };
                    let status = if utxo.coinbase && !utxo.mature {
                        "immature coinbase"
                    } else if utxo.confirmed {
                        "confirmed"
                    } else {
                        "pending"
                    };
                    let confirmation = utxo
                        .confirmation_height
                        .map(|height| height.to_string())
                        .unwrap_or_else(|| "unconfirmed".to_owned());

                    println!();
                    println!("  {}", utxo.outpoint);
                    println!("    Value:        {} sats", utxo.value.to_sat());
                    println!("    Derivation:   {keychain} #{}", utxo.derivation_index);
                    println!("    Status:       {status}");
                    println!("    Block height: {confirmation}");
                    println!("    Locked:       {}", yes_or_no(utxo.locked));
                    println!("    Spendable:    {}", yes_or_no(utxo.spendable));
                }

                Ok(())
            }
            Command::Send {
                to,
                amount,
                fee_rate,
                mnemonic,
            } => {
                let mnemonic = mnemonic
                    .as_deref()
                    .ok_or(WalletError::SigningMnemonicRequired)?;
                let fee_rate = fee_rate_from_sat_per_vbyte(fee_rate)?;
                let mut service = WalletService::load(&self.config, Some(mnemonic))?;
                let node = BitcoinCoreNode::connect(&self.config.rpc, self.config.network)?;
                service.sync(&node)?;

                let summary = service.send(&node, &to, Amount::from_sat(amount), fee_rate)?;

                println!("Transaction broadcast successfully.");
                println!("  Transaction ID: {}", summary.txid);
                println!("  Sent:           {} sats", summary.sent.to_sat());
                println!("  Fee:            {} sats", summary.fee.to_sat());

                Ok(())
            }
        }
    }
}

fn synchronize_unless_offline(
    config: &WalletConfig,
    service: &mut WalletService,
    offline: bool,
) -> WalletResult<()> {
    if !offline {
        let node = BitcoinCoreNode::connect(&config.rpc, config.network)?;
        service.sync(&node)?;
    }

    Ok(())
}

fn fee_rate_from_sat_per_vbyte(sats_per_vbyte: f64) -> WalletResult<FeeRate> {
    let sats_per_kwu = (sats_per_vbyte * 250.0).ceil();
    if !sats_per_kwu.is_finite() || sats_per_kwu > u64::MAX as f64 {
        return Err(WalletError::InvalidFeeRate("value is too large".to_owned()));
    }

    Ok(FeeRate::from_sat_per_kwu(sats_per_kwu as u64))
}

const fn yes_or_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use super::fee_rate_from_sat_per_vbyte;

    #[test]
    fn converts_fractional_sat_per_vbyte_without_rounding_down() {
        let fee_rate = fee_rate_from_sat_per_vbyte(1.5).unwrap();

        assert_eq!(fee_rate.to_sat_per_kwu(), 375);
    }

    #[test]
    fn rejects_fee_rates_too_large_to_represent() {
        assert!(fee_rate_from_sat_per_vbyte(f64::MAX).is_err());
    }
}
