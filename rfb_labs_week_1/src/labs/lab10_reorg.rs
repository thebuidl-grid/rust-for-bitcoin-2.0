//! Lab 10 — observe competing branches and most-work convergence.

use crate::model::{ChainTip, ForkSnapshot, ReorgReport};
use crate::rpc::{parse_cli_value, RpcClient};
use crate::LabResult;

/// Read height, best-block hash, and accumulated chainwork from one node.
pub fn get_chain_tip<C: RpcClient>(client: &C) -> LabResult<ChainTip> {
    // TODO: call getblockchaininfo and decode blocks, bestblockhash, and chainwork.
    // todo!("Lab 10: inspect one node's chain tip")
    let raw = client.call(None, "getblockchaininfo", &[])?;
    let value = parse_cli_value(&raw)?;
    Ok(ChainTip {
        height: value["blocks"].as_u64().unwrap_or_default(),
        best_block_hash: value["bestblockhash"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        chainwork: value["chainwork"].as_str().unwrap_or_default().to_owned(),
    })
}

/// Disconnect a peer by its address.
pub fn disconnect_peer<C: RpcClient>(client: &C, peer_address: &str) -> LabResult<()> {
    // TODO: call disconnectnode with the peer address.
    // todo!("Lab 10: disconnect competing nodes")
    client
        .call(None, "disconnectnode", &[peer_address.to_string()])
        .map(|_| ())
}

/// Reconnect a peer for a one-time synchronization attempt.
pub fn reconnect_peer<C: RpcClient>(client: &C, peer_address: &str) -> LabResult<()> {
    // TODO: call addnode with the address and `onetry`.
    // todo!("Lab 10: reconnect competing nodes")
    client
        .call(
            None,
            "addnode",
            &[peer_address.to_string(), "onetry".to_string()],
        )
        .map(|_| ())
}

/// Compare the private competing tips with the final synchronized tips.
pub fn build_reorg_report(
    common_tip_before_split: &str,
    competing_tips: ForkSnapshot,
    final_tips: ForkSnapshot,
) -> ReorgReport {
    // TODO: nodes converge when their final best hashes and heights match.
    // todo!("Lab 10: report most-work-chain convergence")
    ReorgReport {
        common_tip_before_split: common_tip_before_split.to_owned(),
        competing_tips,
        final_tips: final_tips.clone(),
        converged: final_tips.node_a.best_block_hash == final_tips.node_b.best_block_hash
            && final_tips.node_a.height == final_tips.node_b.height,
    }
}
