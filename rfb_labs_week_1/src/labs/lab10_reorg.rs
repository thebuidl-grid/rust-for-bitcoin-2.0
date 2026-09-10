//! Lab 10 — observe competing branches and most-work convergence.

use crate::model::{ChainTip, ForkSnapshot, ReorgReport};
use crate::rpc::{RpcClient, parse_cli_value, required_string, required_u64};
use crate::LabResult;

/// Read height, best-block hash, and accumulated chainwork from one node.
pub fn get_chain_tip<C: RpcClient>(client: &C) -> LabResult<ChainTip> {
    let raw = client.call(None, "getblockchaininfo", &[])?;

    let block_height = required_u64(&parse_cli_value(&raw)?, "blocks")?;
    let value = parse_cli_value(&raw)?;

    let best_block_hash = required_string(&value, "bestblockhash")?;
    let chainwork = required_string(&value, "chainwork")?;

    Ok(ChainTip {
        height: block_height,
        best_block_hash,
        chainwork,
    })
}

/// Disconnect a peer by its address.
pub fn disconnect_peer<C: RpcClient>(client: &C, peer_address: &str) -> LabResult<()> {
    client.call(None, "disconnectnode", &[peer_address.to_string()])?;
    Ok(())
}

/// Reconnect a peer for a one-time synchronization attempt.
pub fn reconnect_peer<C: RpcClient>(client: &C, peer_address: &str) -> LabResult<()> {
    client.call(None, "addnode", &[peer_address.to_string(), "onetry".to_string()])?;
    Ok(())
}

/// Compare the private competing tips with the final synchronized tips.
pub fn build_reorg_report(
    common_tip_before_split: &str,
    competing_tips: ForkSnapshot,
    final_tips: ForkSnapshot,
) -> ReorgReport {
        let converged = final_tips.node_a.height == final_tips.node_b.height
        && final_tips.node_a.best_block_hash == final_tips.node_b.best_block_hash;

    ReorgReport {
        common_tip_before_split: common_tip_before_split.to_string(),
        competing_tips,
        final_tips,
        converged,
    }}
