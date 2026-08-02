//! Lab 10 — observe competing branches and most-work convergence.

use crate::model::{ChainTip, ForkSnapshot, ReorgReport};
use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::{LabError, LabResult};

/// Read height, best-block hash, and accumulated chainwork from one node.
pub fn get_chain_tip<C: RpcClient>(client: &C) -> LabResult<ChainTip> {
    // 1. Call `getblockchaininfo` on the target node
    let raw = client.call(None, "getblockchaininfo", &[])?;
    let value = parse_cli_value(&raw)?;

    // 2. Decode height (`blocks`), `bestblockhash`, and `chainwork`
    let height = value["blocks"]
        .as_u64()
        .ok_or_else(|| LabError::Parse("getblockchaininfo missing u64 'blocks'".to_owned()))?;

    let best_block_hash = required_string(&value, "bestblockhash")?;
    let chainwork = required_string(&value, "chainwork")?;

    Ok(ChainTip {
        height,
        best_block_hash,
        chainwork,
    })
}

/// Disconnect a peer by its address.
pub fn disconnect_peer<C: RpcClient>(client: &C, peer_address: &str) -> LabResult<()> {
    // Call RPC: disconnectnode "address"
    client.call(None, "disconnectnode", &[peer_address.into()])?;
    Ok(())
}

/// Reconnect a peer for a one-time synchronization attempt.
pub fn reconnect_peer<C: RpcClient>(client: &C, peer_address: &str) -> LabResult<()> {
    // Call RPC: addnode "address" "onetry"
    client.call(None, "addnode", &[peer_address.into(), "onetry".into()])?;
    Ok(())
}

/// Compare the private competing tips with the final synchronized tips.
pub fn build_reorg_report(
    common_tip_before_split: &str,
    competing_tips: ForkSnapshot,
    final_tips: ForkSnapshot,
) -> ReorgReport {
    // 1. Check if both node tips match in height and block hash
    let converged = final_tips.node_a.height == final_tips.node_b.height
        && final_tips.node_a.best_block_hash == final_tips.node_b.best_block_hash;

    ReorgReport {
        common_tip_before_split: common_tip_before_split.to_owned(),
        competing_tips,
        final_tips,
        converged,
    }
}