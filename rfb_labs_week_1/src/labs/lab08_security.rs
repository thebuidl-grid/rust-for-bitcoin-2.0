//! Lab 08 — inspect proof-linked headers and confirmation depth.

use crate::model::{BlockHeaderEvidence, SecurityReport};
use crate::rpc::RpcClient;
use crate::LabResult;
use serde_json::Value;

/// Decode a block header into the fields used by the lab.
pub fn get_block_header<C: RpcClient>(
    client: &C,
    block_hash: &str,
) -> LabResult<BlockHeaderEvidence> {
    let json_str = client.call(None, "getblockheader", &[block_hash.to_string()])?;
    let header: BlockHeaderEvidence = serde_json::from_str(&json_str)?;
    Ok(header)
}

/// Mine an exact number of additional blocks and return their hashes.
pub fn mine_additional_blocks<C: RpcClient>(
    client: &C,
    miner_address: &str,
    count: u64,
) -> LabResult<Vec<String>> {
    let json_str = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), miner_address.to_string()],
    )?;
    let block_hashes: Vec<String> = serde_json::from_str(&json_str)?;
    Ok(block_hashes)
}

/// Read a transaction's confirmation count.
pub fn get_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let json_str = client.call(
        Some(wallet_name),
        "gettransaction",
        &[txid.to_string()],
    )?;
    let val: Value = serde_json::from_str(&json_str)?;
    let confirmations = val["confirmations"].as_i64().unwrap_or(0);
    Ok(confirmations)
}

/// Record the block header and prove one confirmation becomes six after five blocks.
pub fn build_security_report<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
    block_hash: &str,
    miner_address: &str,
) -> LabResult<SecurityReport> {
    let header = get_block_header(client, block_hash)?;
    let confirmations_before = get_confirmations(client, wallet_name, txid)?;
    mine_additional_blocks(client, miner_address, 5)?;
    let confirmations_after = get_confirmations(client, wallet_name, txid)?;

    Ok(SecurityReport {
        header,
        confirmations_before,
        confirmations_after,
    })
}