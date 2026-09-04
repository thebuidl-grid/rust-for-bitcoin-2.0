/// Bitcoin Core RPC helpers.
///
/// Wraps `bitcoincore_rpc::Client` for two tasks:
///   1. Sync wallet state via `bdk_bitcoind_rpc::Emitter`
///   2. Broadcast a signed transaction via `sendrawtransaction`
use anyhow::{Context, Result};
use bdk_bitcoind_rpc::{
    bitcoincore_rpc::{Auth, Client as RpcClient, RpcApi},
    Emitter,
};
use bdk_wallet::{
    bitcoin::{consensus, Transaction},
    PersistedWallet,
};
use log::info;

use crate::config::Config;

/// Build an authenticated RPC client from config.
pub fn client(cfg: &Config) -> Result<RpcClient> {
    let auth = Auth::UserPass(cfg.rpc_user.clone(), cfg.rpc_pass.clone());
    RpcClient::new(&cfg.rpc_url, auth)
        .with_context(|| format!("Cannot connect to Bitcoin Core at {}", cfg.rpc_url))
}

/// Sync the BDK wallet against Bitcoin Core using `bdk_bitcoind_rpc::Emitter`.
///
/// Walks every new block from the wallet's current tip, applies each to the
/// wallet, then ingests the mempool.  Persists any changes before returning.
pub fn sync(
    rpc: &RpcClient,
    wallet: &mut PersistedWallet<bdk_wallet::rusqlite::Connection>,
    conn: &mut bdk_wallet::rusqlite::Connection,
) -> Result<()> {
    let wallet_tip = wallet.latest_checkpoint();
    info!("Wallet tip before sync: height {}", wallet_tip.height());

    // Collect unconfirmed transactions already known to the wallet so the
    // Emitter can avoid re-emitting them as "new" mempool entries.
    let unconfirmed: Vec<_> = wallet
        .transactions()
        .filter(|tx| tx.chain_position.is_unconfirmed())
        .map(|tx| tx.tx_node.tx.clone())
        .collect();

    let mut emitter = Emitter::new(rpc, wallet_tip, 0, unconfirmed);

    let mut blocks_applied = 0u32;
    while let Some(block_event) = emitter.next_block().context("Error fetching next block")? {
        let height = block_event.block_height();
        let connected_to = block_event.connected_to();
        wallet
            .apply_block_connected_to(&block_event.block, height, connected_to)
            .context("Error applying block to wallet")?;
        wallet
            .persist(conn)
            .context("Failed to persist after block")?;
        blocks_applied += 1;
    }

    // Apply mempool (new and evicted transactions).
    let mempool_event = emitter.mempool().context("Error fetching mempool")?;
    wallet.apply_evicted_txs(mempool_event.evicted);
    wallet.apply_unconfirmed_txs(mempool_event.update);
    wallet
        .persist(conn)
        .context("Failed to persist wallet state")?;

    info!("Sync complete — {} new block(s) applied.", blocks_applied);
    Ok(())
}

/// Broadcast a signed transaction via `sendrawtransaction`.
///
/// Returns the txid as a hex string.
pub fn broadcast(rpc: &RpcClient, tx: &Transaction) -> Result<String> {
    let raw = consensus::serialize(tx);
    let txid = rpc
        .send_raw_transaction(raw.as_slice())
        .context("Failed to broadcast transaction")?;
    Ok(txid.to_string())
}
