/// Business logic for each CLI command.
///
/// Each function:
///   1. Loads config from env
///   2. Opens the wallet (SQLite-persisted)
///   3. Creates an RPC client where needed
///   4. Does the work
///   5. Persists any state changes
use anyhow::{Context, Result};
use bdk_bitcoind_rpc::bitcoincore_rpc::RpcApi;
use bdk_wallet::{
    bitcoin::{Address, Amount, FeeRate, Network, Sequence},
    KeychainKind, SignOptions,
};
use log::info;
use std::str::FromStr;

use crate::{config::Config, node, wallet as wallet_mod};

// ─── address ─────────────────────────────────────────────────────────────────

/// Reveal the next unused external (receiving) address.
pub fn get_address() -> Result<()> {
    let cfg = Config::from_env()?;
    let (mut wallet, mut conn) = wallet_mod::open(&cfg)?;

    let info = wallet.reveal_next_address(KeychainKind::External);
    wallet
        .persist(&mut conn)
        .context("Failed to persist wallet")?;

    println!("Receive address #{}: {}", info.index, info.address);
    Ok(())
}

// ─── balance ─────────────────────────────────────────────────────────────────

/// Print confirmed and unconfirmed balance.
pub fn get_balance() -> Result<()> {
    let cfg = Config::from_env()?;
    let (wallet, _conn) = wallet_mod::open(&cfg)?;

    let balance = wallet.balance();
    println!(
        "Balance:\n  Confirmed:         {} sats\n  Trusted pending:   {} sats\n  Untrusted pending: {} sats\n  Total:             {} sats",
        balance.confirmed.to_sat(),
        balance.trusted_pending.to_sat(),
        balance.untrusted_pending.to_sat(),
        balance.total().to_sat(),
    );
    Ok(())
}

// ─── sync ────────────────────────────────────────────────────────────────────

/// Sync wallet state against Bitcoin Core.
pub fn sync() -> Result<()> {
    let cfg = Config::from_env()?;
    let (mut wallet, mut conn) = wallet_mod::open(&cfg)?;
    let rpc = node::client(&cfg)?;

    info!("Starting sync against {}", cfg.rpc_url);
    node::sync(&rpc, &mut wallet, &mut conn)?;

    let balance = wallet.balance();
    println!(
        "Sync complete.\nBalance: {} confirmed sats / {} unconfirmed sats",
        balance.confirmed.to_sat(),
        (balance.trusted_pending + balance.untrusted_pending).to_sat(),
    );
    Ok(())
}

// ─── send ────────────────────────────────────────────────────────────────────

/// Build, sign, and broadcast a transaction.
pub fn send(to: &str, amount_sats: u64) -> Result<()> {
    let cfg = Config::from_env()?;
    let (mut wallet, mut conn) = wallet_mod::open(&cfg)?;
    let rpc = node::client(&cfg)?;

    // Sync first so coin selection has up-to-date UTXOs.
    info!("Syncing before send…");
    node::sync(&rpc, &mut wallet, &mut conn)?;

    let recipient = Address::from_str(to)
        .with_context(|| format!("Invalid destination address '{to}'"))?
        .require_network(Network::Regtest)
        .with_context(|| format!("Address '{to}' is not valid on regtest"))?;

    let amount = Amount::from_sat(amount_sats);

    // Estimate feerate: ask Core for a 6-block target; fall back to 1 sat/vbyte.
    let fee_rate = match rpc.estimate_smart_fee(6, None) {
        Ok(est) => {
            if let Some(btc_per_kvb) = est.fee_rate {
                // btc_per_kvb is BTC/kVB; convert to sat/vbyte (ceiling)
                let sat_per_vb = (btc_per_kvb.to_btc() * 1e8 / 1000.0).ceil() as u64;
                FeeRate::from_sat_per_vb(sat_per_vb.max(1)).unwrap_or(FeeRate::BROADCAST_MIN)
            } else {
                FeeRate::BROADCAST_MIN
            }
        }
        Err(_) => FeeRate::BROADCAST_MIN,
    };

    info!(
        "Using fee rate: {} sat/vbyte",
        fee_rate.to_sat_per_vb_ceil()
    );

    // Build the transaction.
    // RBF is signalled by setting nSequence < 0xFFFFFFFE on inputs.
    // BDK 3.x exposes this via `set_exact_sequence`.
    let mut tx_builder = wallet.build_tx();
    tx_builder
        .add_recipient(recipient.script_pubkey(), amount)
        .fee_rate(fee_rate)
        // Signal RBF (BIP-125): nSequence = 0xFFFFFFFD
        .set_exact_sequence(Sequence::ENABLE_RBF_NO_LOCKTIME);

    let mut psbt = tx_builder
        .finish()
        .context("Failed to build transaction — check balance and UTXOs")?;

    // Sign the PSBT with the wallet's internal keys.
    wallet
        .sign(&mut psbt, SignOptions::default())
        .context("Failed to sign transaction")?;

    // Extract the final transaction.
    let tx = psbt
        .extract_tx()
        .context("Failed to extract signed transaction from PSBT")?;

    let txid = node::broadcast(&rpc, &tx)?;

    // Persist updated wallet state (spent UTXOs, change address index bump).
    wallet
        .persist(&mut conn)
        .context("Failed to persist wallet")?;

    println!("✅  Transaction broadcast!\n   txid: {txid}");
    println!("   Sent {} sats to {to}", amount_sats);
    Ok(())
}
