use bdk_bitcoind_rpc::Emitter;

use crate::error::WalletError;
use crate::node::Node;
use crate::wallet::Wallet;

/// Result of one sync pass, for reporting to the user.
pub struct SyncReport {
    pub blocks_applied: u32,
    pub tip_height: u32,
    pub tip_hash: String,
}

/// Pull blocks and mempool transactions from Bitcoin Core and fold them into the
/// wallet, then persist.
///
/// This uses `bdk_bitcoind_rpc::Emitter`, which walks the chain from the
/// wallet's last checkpoint using only public RPC calls (`getblock`,
/// `getrawmempool`, ...). It does not touch Core's own wallet, so Core can run
/// with `-disablewallet`.
pub fn run(wallet: &mut Wallet, node: &Node, start_height: u32) -> Result<SyncReport, WalletError> {
    let checkpoint = wallet.inner().latest_checkpoint();

    let mut emitter = Emitter::new(
        node.client(),
        checkpoint,
        start_height,
        wallet
            .inner()
            .transactions()
            .filter(|tx| tx.chain_position.is_unconfirmed()),
    );

    let mut blocks_applied = 0u32;
    while let Some(event) = emitter
        .next_block()
        .map_err(|e| WalletError::Connect(e.to_string()))?
    {
        let height = event.block_height();
        let connected_to = event.connected_to();
        wallet
            .inner_mut()
            .apply_block_connected_to(&event.block, height, connected_to)
            .map_err(|e| WalletError::Connect(e.to_string()))?;

        blocks_applied += 1;
        if blocks_applied.is_multiple_of(500) {
            wallet.persist()?;
        }
    }

    let mempool = emitter
        .mempool()
        .map_err(|e| WalletError::Connect(e.to_string()))?;
    wallet.inner_mut().apply_evicted_txs(mempool.evicted);
    wallet.inner_mut().apply_unconfirmed_txs(mempool.update);

    wallet.persist()?;

    let tip = wallet.inner().latest_checkpoint();
    return Ok(SyncReport {
        blocks_applied,
        tip_height: tip.height(),
        tip_hash: tip.hash().to_string(),
    });
}
