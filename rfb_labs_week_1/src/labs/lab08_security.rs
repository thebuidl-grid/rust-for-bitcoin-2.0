//! Lab 08 — inspect proof-linked headers and confirmation depth.

use crate::model::{BlockHeaderEvidence, SecurityReport};
use crate::rpc::RpcClient;
use crate::LabResult;

/// Decode a block header into the fields used by the lab.
pub fn get_block_header<C: RpcClient>(
    client: &C,
    block_hash: &str,
) -> LabResult<BlockHeaderEvidence> {
    let raw = client.call(None, "getblockheader", &[block_hash.to_owned()])?;
    let value = crate::rpc::parse_cli_value(&raw)?;

    let hash = crate::rpc::required_string(&value, "hash")?;
    let height = crate::rpc::required_u64(&value, "height")?;
    let previous_block_hash = value
        .get("previousblockhash")
        .or_else(|| value.get("previous_block_hash"))
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned);
    let merkle_root = value
        .get("merkleroot")
        .or_else(|| value.get("merkle_root"))
        .and_then(serde_json::Value::as_str)
        .ok_or(crate::LabError::MissingField("merkleroot"))?
        .to_owned();
    let nonce = crate::rpc::required_u64(&value, "nonce")?;
    let difficulty = crate::rpc::required_f64(&value, "difficulty")?;
    let bits = crate::rpc::required_string(&value, "bits")?;
    let confirmations = value
        .get("confirmations")
        .and_then(serde_json::Value::as_i64)
        .ok_or(crate::LabError::MissingField("confirmations"))?;
    let chainwork = crate::rpc::required_string(&value, "chainwork")?;

    Ok(BlockHeaderEvidence {
        hash,
        height,
        previous_block_hash,
        merkle_root,
        nonce,
        difficulty,
        bits,
        confirmations,
        chainwork,
    })
}

/// Mine an exact number of additional blocks and return their hashes.
pub fn mine_additional_blocks<C: RpcClient>(
    client: &C,
    miner_address: &str,
    count: u64,
) -> LabResult<Vec<String>> {
    crate::labs::lab03_maturity::mine_blocks(client, miner_address, count)
}

/// Read a transaction's confirmation count.
pub fn get_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    crate::labs::lab07_confirm::transaction_confirmations(client, wallet_name, txid)
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
