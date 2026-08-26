//! Lab 10 — observe competing branches and most-work convergence.

use crate::model::{ChainTip, ForkSnapshot, ReorgReport};
use crate::rpc::RpcClient;
use crate::{LabResult, LabError};
use serde::Deserialize;

#[derive(Deserialize)]
struct RawChainInfo {
    blocks: u64,
    #[serde(rename = "bestblockhash")]
    best_block_hash: String,
    chainwork: String,
}

/// Read height, best-block hash, and accumulated chainwork from one node.
pub fn get_chain_tip<C: RpcClient>(client: &C) -> LabResult<ChainTip> {
    // TODO: call getblockchaininfo and decode blocks, bestblockhash, and chainwork.
    // todo!("Lab 10: inspect one node's chain tip")
    let raw = client.call(None, "getblockchaininfo", &[])?;
    let parsed: RawChainInfo =
        serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    Ok(ChainTip {
        height: parsed.blocks,
        best_block_hash: parsed.best_block_hash,
        chainwork: parsed.chainwork,
    })
}

/// Disconnect a peer by its address.
pub fn disconnect_peer<C: RpcClient>(client: &C, peer_address: &str) -> LabResult<()> {
    // TODO: call disconnectnode with the peer address.
    // todo!("Lab 10: disconnect competing nodes")
     client.call(None, "disconnectnode", &[peer_address.to_string()])?;
    Ok(())
}

/// Reconnect a peer for a one-time synchronization attempt.
pub fn reconnect_peer<C: RpcClient>(client: &C, peer_address: &str) -> LabResult<()> {
    // TODO: call addnode with the address and `onetry`.
    // todo!("Lab 10: reconnect competing nodes")
    

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
    // TODO: nodes converge when their final best hashes and heights match.
    // todo!("Lab 10: report most-work-chain convergence")
    let converged = final_tips.node_a.best_block_hash == final_tips.node_b.best_block_hash
        && final_tips.node_a.height == final_tips.node_b.height;

    ReorgReport {
        common_tip_before_split: common_tip_before_split.to_string(),
        competing_tips,
        final_tips,
        converged,
    }
}
