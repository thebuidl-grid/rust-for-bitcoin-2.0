//! Lab 08 — inspect proof-linked headers and confirmation depth.

use crate::model::{BlockHeaderEvidence, SecurityReport};
use crate::rpc::{required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};

/// Decode a block header into the fields used by the lab.
pub fn get_block_header<C: RpcClient>(
    client: &C,
    block_hash: &str,
) -> LabResult<BlockHeaderEvidence> {
    let response = client.call(None, "getblockheader", &[block_hash.to_string()])?;
    let header: serde_json::Value = serde_json::from_str(&response)?;

    let hash = required_string(&header, "hash")?;
    let height = required_u64(&header, "height")?;
    let previous_block_hash = optional_string(&header, "previousblockhash")?;
    let merkle_root = required_string(&header, "merkleroot")?;
    let nonce = required_u64(&header, "nonce")?;
    let difficulty = required_f64(&header, "difficulty")?;
    let bits = required_string(&header, "bits")?;
    let confirmations = header
        .get("confirmations")
        .and_then(serde_json::Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))?;
    let chainwork = required_string(&header, "chainwork")?;

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
    let response = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), miner_address.to_string()],
    )?;
    let block_hashes = serde_json::from_str::<Vec<String>>(&response)?;

    Ok(block_hashes)
}

/// Read a transaction's confirmation count.
pub fn get_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let response = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let transaction: serde_json::Value = serde_json::from_str(&response)?;
    let confirmations = transaction
        .get("confirmations")
        .and_then(serde_json::Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))?;

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

fn optional_string(value: &serde_json::Value, field: &'static str) -> LabResult<Option<String>> {
    let Some(field_value) = value.get(field) else {
        return Ok(None);
    };
    let field_value = field_value
        .as_str()
        .ok_or_else(|| LabError::Parse(format!("invalid `{field}` field")))?;

    Ok(Some(field_value.to_owned()))
}
