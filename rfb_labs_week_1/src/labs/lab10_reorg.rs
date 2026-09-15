//! Lab 10 — observe competing branches and most-work convergence.

use crate::model::{ChainTip, ForkSnapshot, ReorgReport};
use crate::rpc::{parse_cli_value, required_string, required_u64, RpcClient};
use crate::LabResult;

/// Read height, best-block hash, and accumulated chainwork from one node.
pub fn get_chain_tip<C: RpcClient>(client: &C) -> LabResult<ChainTip> {
    let raw = client.call(None, "getblockchaininfo", &[])?;
    let value = parse_cli_value(&raw)?;

    Ok(ChainTip {
        height: required_u64(&value, "blocks")?,
        best_block_hash: required_string(&value, "bestblockhash")?,
        chainwork: required_string(&value, "chainwork")?,
    })
}

/// Disconnect a peer by its address.
pub fn disconnect_peer<C: RpcClient>(client: &C, peer_address: &str) -> LabResult<()> {
    client.call(None, "disconnectnode", &[peer_address.to_owned()])?;
    Ok(())
}

/// Reconnect a peer for a one-time synchronization attempt.
pub fn reconnect_peer<C: RpcClient>(client: &C, peer_address: &str) -> LabResult<()> {
    client.call(
        None,
        "addnode",
        &[peer_address.to_owned(), "onetry".to_owned()],
    )?;
    Ok(())
}

/// Compare the private competing tips with the final synchronized tips.
pub fn build_reorg_report(
    common_tip_before_split: &str,
    competing_tips: ForkSnapshot,
    final_tips: ForkSnapshot,
) -> ReorgReport {
    let converged = final_tips.node_a == final_tips.node_b;

    ReorgReport {
        common_tip_before_split: common_tip_before_split.to_owned(),
        competing_tips,
        final_tips,
        converged,
    }
}
