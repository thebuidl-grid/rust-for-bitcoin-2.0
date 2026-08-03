use crate::model::{ChainTip, ForkSnapshot, ReorgReport};
use crate::rpc::{parse_cli_value, required_string, required_u64, RpcClient};
use crate::LabResult;

pub fn get_chain_tip<C: RpcClient>(client: &C) -> LabResult<ChainTip> {
    let raw = client.call(None, "getblockchaininfo", &[])?;
    let value = parse_cli_value(&raw)?;

    Ok(ChainTip {
        height: required_u64(&value, "blocks")?,
        best_block_hash: required_string(&value, "bestblockhash")?,
        chainwork: required_string(&value, "chainwork")?,
    })
}

pub fn disconnect_peer<C: RpcClient>(client: &C, peer_address: &str) -> LabResult<()> {
    client.call(None, "disconnectnode", &[peer_address.to_owned()])?;
    Ok(())
}

pub fn reconnect_peer<C: RpcClient>(client: &C, peer_address: &str) -> LabResult<()> {
    client.call(
        None,
        "addnode",
        &[peer_address.to_owned(), "onetry".to_owned()],
    )?;
    Ok(())
}

pub fn build_reorg_report(
    common_tip_before_split: &str,
    competing_tips: ForkSnapshot,
    final_tips: ForkSnapshot,
) -> ReorgReport {
    let converged = final_tips.node_a.best_block_hash == final_tips.node_b.best_block_hash
        && final_tips.node_a.height == final_tips.node_b.height;

    ReorgReport {
        common_tip_before_split: common_tip_before_split.to_owned(),
        competing_tips,
        final_tips,
        converged,
    }
}
