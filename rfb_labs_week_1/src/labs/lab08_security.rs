//! Lab 08 — inspect proof-linked headers and confirmation depth.

use crate::model::{BlockHeaderEvidence, SecurityReport};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

/// Decode a block header into the fields used by the lab.
pub fn get_block_header<C: RpcClient>(
    client: &C,
    block_hash: &str,
) -> LabResult<BlockHeaderEvidence> {
    // Call getblockheader with just the block hash parameter
    let raw = client.call(None, "getblockheader", &[block_hash.to_string()])?;
    let val = parse_cli_value(&raw)?;

    let hash = required_string(&val, "hash")?;
    let height = required_u64(&val, "height")?;
    let merkle_root = required_string(&val, "merkleroot")?;
    let nonce = required_u64(&val, "nonce")?;
    let difficulty = required_f64(&val, "difficulty")?;
    let bits = required_string(&val, "bits")?;
    let chainwork = required_string(&val, "chainwork")?;
    let confirmations = val
        .get("confirmations")
        .and_then(Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))?;

    let previous_block_hash = val
        .get("previousblockhash")
        .and_then(|v| v.as_str())
        .map(ToOwned::to_owned);

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
    let raw = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), miner_address.to_string()],
    )?;
    let val = parse_cli_value(&raw)?;

    let hashes = val.as_array().ok_or_else(|| {
        LabError::Parse("expected array of block hashes from generatetoaddress".to_string())
    })?;

    let mut result = Vec::new();
    for item in hashes {
        if let Some(h) = item.as_str() {
            result.push(h.to_string());
        }
    }

    Ok(result)
}

/// Read a transaction's confirmation count.
pub fn get_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let val = parse_cli_value(&raw)?;

    let confirmations = val
        .get("confirmations")
        .and_then(Value::as_i64)
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
    // 1. Fetch header details for the initial block
    let header = get_block_header(client, block_hash)?;

    // 2. Read initial confirmations
    let confirmations_before = get_confirmations(client, wallet_name, txid)?;

    // 3. Mine 5 additional blocks
    let _added_hashes = mine_additional_blocks(client, miner_address, 5)?;

    // 4. Read final confirmations after mining
    let confirmations_after = get_confirmations(client, wallet_name, txid)?;

    Ok(SecurityReport {
        header,
        confirmations_before,
        confirmations_after,
    })
}
