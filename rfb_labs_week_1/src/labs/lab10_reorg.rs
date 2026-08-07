//! Lab 10 — observe competing branches and most-work convergence.

use crate::model::{ChainTip, ForkSnapshot, ReorgReport};
use crate::rpc::RpcClient;
use crate::LabResult;
use serde::Deserialize;

#[derive(Deserialize)]
struct BlockchainInfo {
    blocks: u64,
    bestblockhash: String,
    chainwork: String,
}

/// Read height, best-block hash, and accumulated chainwork from one node.
pub fn get_chain_tip<C: RpcClient>(client: &C) -> LabResult<ChainTip> {
    let json_str = client.call(None, "getblockchaininfo", &[])?;
    let info: BlockchainInfo = serde_json::from_str(&json_str)?;

    Ok(ChainTip {
        height: info.blocks,
        best_block_hash: info.bestblockhash,
        chainwork: info.chainwork,
    })
}

/// Disconnect a peer by its address.
pub fn disconnect_peer<C: RpcClient>(client: &C, peer_address: &str) -> LabResult<()> {
    client.call(None, "disconnectnode", &[peer_address.to_string()])?;
    Ok(())
}

/// Reconnect a peer for a one-time synchronization attempt.
pub fn reconnect_peer<C: RpcClient>(client: &C, peer_address: &str) -> LabResult<()> {
    client.call(
        None,
        "addnode",
        &[peer_address.to_string(), "onetry".to_string()],
    )?;
    Ok(())
}

/// Compare the private competing tips with the final synchronized tips.
pub fn build_reorg_report(
    common_tip_before_split: &str,
    competing_tips: ForkSnapshot,
    final_tips: ForkSnapshot,
) -> ReorgReport {
    let converged = final_tips.node_a.best_block_hash == final_tips.node_b.best_block_hash
        && final_tips.node_a.height == final_tips.node_b.height;

    ReorgReport {
        common_tip_before_split: common_tip_before_split.to_string(),
        competing_tips,
        final_tips,
        converged,
    }
}