mod config;
mod keys;
mod node;
mod raw_bitcoin;
mod wallet;

use anyhow::{Context, Result};
use bdk_wallet::bitcoin::Network;
use clap::Parser;
use std::str::FromStr;

use config::{Cli, CliDescriptorType, Commands};
use keys::{DescriptorType, WalletKeys};
use node::NodeClient;
use raw_bitcoin::RawBitcoinDemo;
use wallet::WalletManager;

fn parse_network(net_str: &str) -> Result<Network> {
    match net_str.to_lowercase().as_str() {
        "regtest" => Ok(Network::Regtest),
        "testnet" => Ok(Network::Testnet),
        "signet" => Ok(Network::Signet),
        "bitcoin" | "mainnet" => Ok(Network::Bitcoin),
        _ => Network::from_str(net_str).context("Invalid Bitcoin network specified"),
    }
}

fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();

    let network = parse_network(&cli.network)?;
    let desc_type = match cli.descriptor_type {
        CliDescriptorType::Wpkh => DescriptorType::Wpkh,
        CliDescriptorType::Taproot => DescriptorType::Taproot,
    };

    match cli.command {
        Commands::GenerateMnemonic => {
            let keys = WalletKeys::generate_new(network, desc_type)?;
            println!("=== Generated New Wallet Keys ===");
            println!("Mnemonic (BIP39): {}", keys.mnemonic);
            println!("Descriptor Type: {:?}", keys.descriptor_type);
            println!("External Descriptor: {}", keys.external_descriptor);
            println!("Internal Descriptor: {}", keys.internal_descriptor);
            println!("\nTip: Save the mnemonic or pass it via --mnemonic or MNEMONIC env var.");
        }
        Commands::Address { change } => {
            let mnemonic = cli
                .mnemonic
                .context("Mnemonic required! Pass --mnemonic or set MNEMONIC env var.")?;
            let keys = WalletKeys::from_mnemonic(&mnemonic, network, desc_type)?;
            let mut wallet_mgr = WalletManager::open_or_create(
                &cli.db_path,
                &keys.external_descriptor,
                Some(&keys.internal_descriptor),
                network,
            )?;

            let addr = if change {
                wallet_mgr.get_change_address()?
            } else {
                wallet_mgr.get_new_address()?
            };

            let addr_type = if change { "Change (Internal)" } else { "Receiving (External)" };
            println!("New {} Address: {}", addr_type, addr);
        }
        Commands::Balance => {
            let mnemonic = cli
                .mnemonic
                .context("Mnemonic required! Pass --mnemonic or set MNEMONIC env var.")?;
            let keys = WalletKeys::from_mnemonic(&mnemonic, network, desc_type)?;
            let wallet_mgr = WalletManager::open_or_create(
                &cli.db_path,
                &keys.external_descriptor,
                Some(&keys.internal_descriptor),
                network,
            )?;

            let bal = wallet_mgr.balance();
            println!("=== Wallet Balance ({}) ===", cli.network);
            println!("Confirmed:         {} sats", bal.confirmed);
            println!("Untrusted Pending: {} sats", bal.untrusted_pending);
            println!("Trusted Pending:   {} sats", bal.trusted_pending);
            println!("Immature:          {} sats", bal.immature);
            println!("Total Balance:     {} sats", bal.total());
        }
        Commands::Utxos => {
            let mnemonic = cli
                .mnemonic
                .context("Mnemonic required! Pass --mnemonic or set MNEMONIC env var.")?;
            let keys = WalletKeys::from_mnemonic(&mnemonic, network, desc_type)?;
            let wallet_mgr = WalletManager::open_or_create(
                &cli.db_path,
                &keys.external_descriptor,
                Some(&keys.internal_descriptor),
                network,
            )?;

            let utxos = wallet_mgr.list_utxos();
            println!("=== Unspent Transaction Outputs (UTXOs) ===");
            if utxos.is_empty() {
                println!("No UTXOs found.");
            } else {
                for (idx, utxo) in utxos.iter().enumerate() {
                    println!(
                        "[{}] Outpoint: {} | Value: {} sats | Keychain: {:?}",
                        idx + 1,
                        utxo.outpoint,
                        utxo.txout.value,
                        utxo.keychain
                    );
                }
            }
        }
        Commands::Sync { start_height } => {
            let mnemonic = cli
                .mnemonic
                .context("Mnemonic required! Pass --mnemonic or set MNEMONIC env var.")?;
            let keys = WalletKeys::from_mnemonic(&mnemonic, network, desc_type)?;
            let mut wallet_mgr = WalletManager::open_or_create(
                &cli.db_path,
                &keys.external_descriptor,
                Some(&keys.internal_descriptor),
                network,
            )?;

            println!("Connecting to Bitcoin Core RPC at {}...", cli.rpc_url);
            let node = NodeClient::new(&cli.rpc_url, cli.rpc_user, cli.rpc_pass)?;

            let info = node.get_info()?;
            println!("Connected to node! Chain: {}, Blocks: {}", info.chain, info.blocks);

            println!("Syncing wallet state from height {}...", start_height);
            let blocks_synced = node.sync(&mut wallet_mgr, start_height)?;
            println!("Successfully applied {} new blocks to wallet!", blocks_synced);

            let bal = wallet_mgr.balance();
            println!("Updated Total Balance: {} sats", bal.total());
        }
        Commands::Send {
            recipient,
            amount_sats,
            broadcast,
        } => {
            let mnemonic = cli
                .mnemonic
                .context("Mnemonic required! Pass --mnemonic or set MNEMONIC env var.")?;
            let keys = WalletKeys::from_mnemonic(&mnemonic, network, desc_type)?;
            let mut wallet_mgr = WalletManager::open_or_create(
                &cli.db_path,
                &keys.external_descriptor,
                Some(&keys.internal_descriptor),
                network,
            )?;

            println!(
                "Constructing & signing transaction sending {} sats to {}...",
                amount_sats, recipient
            );
            let tx = wallet_mgr.create_and_sign_tx(&recipient, amount_sats)?;
            let txid = tx.compute_txid();
            println!("Signed Transaction ID (txid): {}", txid);

            if broadcast {
                println!("Broadcasting transaction via Bitcoin Core RPC...");
                let node = NodeClient::new(&cli.rpc_url, cli.rpc_user, cli.rpc_pass)?;
                let broadcast_txid = node.broadcast(&tx)?;
                println!("Successfully broadcast transaction! TXID: {}", broadcast_txid);
            } else {
                println!("Transaction created & signed successfully (not broadcast).");
            }
        }
        Commands::RawDemo { message } => {
            println!("=== Raw rust-bitcoin Script & PSBT Demonstration ===");
            let txout = RawBitcoinDemo::build_op_return_output(message.as_bytes())?;
            println!("Constructed OP_RETURN TxOut:");
            println!("  Value: {} sats", txout.value);
            println!("  ScriptPubKey Hex: {}", txout.script_pubkey.to_hex_string());
            println!("  Is OP_RETURN: {}", txout.script_pubkey.is_op_return());
            println!("\nThis demonstrates building raw scripts for custom protocols (anchors, timestamps) directly with `rust-bitcoin` when BDK's standard payment abstractions are not applicable.");
        }
    }

    Ok(())
}
