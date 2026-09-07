use anyhow::Result;
use bdk_wallet::bitcoin::constants::genesis_block;
use bdk_wallet::bitcoin::BlockHash;
use bdk_wallet::chain::BlockId;

use crate::bitrpc::BitRpcClient;
use crate::wallet::SqliteWallet;


const FIRST_SYNC_LOOKBACK: u32 = 6;


pub fn sync_wallet(wallet: &mut SqliteWallet, client: &BitRpcClient) -> Result<()> {
    let tip_height = client.get_block_count()?;
    let current_height = wallet.latest_checkpoint().height();

    // A fresh wallet's local chain contains only its genesis checkpoint
    // (height 0). Seed it at (tip - lookback) instead of walking the
    // entire mainnet history block by block.
    let mut next_height = if current_height == 0 {
        let start_height = tip_height.saturating_sub(FIRST_SYNC_LOOKBACK);
        seed_at_height(wallet, client, start_height)?;
        start_height + 1
    } else {
        current_height + 1
    };

    while next_height <= tip_height {
        let hash = client.get_block_hash(next_height)?;
        let block = client.get_block(&hash)?;
        wallet.apply_block(&block, next_height)?;
        next_height += 1;
    }

    // Mempool (unconfirmed tx) sync is intentionally skipped -- BitRPC
    // doesn't allow `getrawmempool`. See module docs.

    Ok(())
}


fn seed_at_height(wallet: &mut SqliteWallet, client: &BitRpcClient, height: u32) -> Result<()> {
    let hash = client.get_block_hash(height)?;
    let block  = client.get_block(&hash)?;

    let genesis_hash: BlockHash = genesis_block(wallet.network()).block_hash();
    let connected_to = BlockId { height: 0, hash: genesis_hash };

    wallet.apply_block_connected_to(&block, height, connected_to)?;
    Ok(())
}