//! Chain sync driven by `bdk_bitcoind_rpc::Emitter`.
//!
//! The emitter walks the node forward block by block from wherever the wallet's
//! checkpoint left off, so a second `sync` after a first one is cheap: it picks
//! up at the stored tip instead of rescanning. It also handles the awkward part
//! of talking to a node directly, which is reorgs. Each emitted block carries
//! the block it connects to, and `apply_block_connected_to` rejects anything
//! that does not line up with what the wallet already believes.

use bdk_bitcoind_rpc::Emitter;
use bitcoincore_rpc::Client;

use crate::error::{Error, Result};
use crate::wallet::WalletHandle;

/// Persist to SQLite every this many blocks, so a long scan that is interrupted
/// does not have to start over.
const PERSIST_EVERY: u32 = 200;

#[derive(Debug, Default)]
pub struct SyncSummary {
    pub blocks_applied: u32,
    pub start_height: u32,
    pub tip_height: u32,
    pub mempool_txs: usize,
    pub evicted_txs: usize,
}

pub fn sync(handle: &mut WalletHandle, client: &Client, start_height: u32) -> Result<SyncSummary> {
    let wallet_tip = handle.wallet.latest_checkpoint();
    let start_from = wallet_tip.height();

    // The emitter needs to know which of our transactions are still unconfirmed
    // so it can tell us if the node drops one from its mempool. Collect them
    // first: the iterator borrows the wallet, and we need it mutably below.
    let unconfirmed: Vec<_> = handle
        .wallet
        .transactions()
        .filter(|tx| !tx.chain_position.is_confirmed())
        .map(|tx| tx.tx_node.tx.clone())
        .collect();

    let mut emitter = Emitter::new(client, wallet_tip, start_height, unconfirmed);
    let mut summary = SyncSummary {
        start_height: start_from,
        ..Default::default()
    };

    while let Some(event) = emitter.next_block()? {
        let height = event.block_height();
        handle
            .wallet
            .apply_block_connected_to(&event.block, height, event.connected_to())
            .map_err(|e| Error::wallet("applying a block to the wallet", e))?;

        summary.blocks_applied += 1;
        summary.tip_height = height;

        if summary.blocks_applied % PERSIST_EVERY == 0 {
            handle.persist()?;
            tracing::debug!(height, "checkpointed sync progress");
        }
    }

    // Mempool last, so unconfirmed transactions are evaluated against the tip we
    // just caught up to.
    let mempool = emitter.mempool()?;
    summary.mempool_txs = mempool.update.len();
    summary.evicted_txs = mempool.evicted.len();
    handle.wallet.apply_evicted_txs(mempool.evicted);
    handle.wallet.apply_unconfirmed_txs(mempool.update);

    handle.persist()?;

    if summary.tip_height == 0 {
        summary.tip_height = handle.wallet.latest_checkpoint().height();
    }

    Ok(summary)
}
