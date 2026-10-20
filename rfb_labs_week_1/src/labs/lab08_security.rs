//! Lab 08 — inspect proof-linked headers and confirmation depth.

use crate::model::{BlockHeaderEvidence, SecurityReport};
use crate::rpc::RpcClient;
use crate::LabResult;

use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64};
use crate::LabError;
use serde_json::Value;

/// Decode a block header into the fields used by the lab.
pub fn get_block_header<C: RpcClient>(
    client: &C,
    block_hash: &str,
) -> LabResult<BlockHeaderEvidence> {
    let raw = client.call(None, "getblockheader", &[block_hash.to_owned()])?;
    let value = parse_cli_value(&raw)?;
    let hash = required_string(&value, "hash")?;
    let height = required_u64(&value, "height")?;
    let previous_block_hash = value
        .get("previousblockhash")
        .and_then(Value::as_str)
        .map(String::from);
    let merkle_root = required_string(&value, "merkleroot")?;
    let nonce = required_u64(&value, "nonce")?;
    let difficulty = required_f64(&value, "difficulty")?;
    let bits = required_string(&value, "bits")?;
    let confirmations = value
        .get("confirmations")
        .and_then(Value::as_i64)
        .ok_or_else(|| LabError::MissingField("confirmations"))?;
    let chainwork = required_string(&value, "chainwork")?;
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
        &[count.to_string(), miner_address.to_owned()],
    )?;
    let value = parse_cli_value(&raw)?;
    let arr = value
        .as_array()
        .ok_or_else(|| LabError::Parse("Expected array of block hashes".to_owned()))?;
    let hashes = arr
        .iter()
        .map(|v| {
            v.as_str()
                .map(String::from)
                .ok_or_else(|| LabError::Parse("Expected block hash to be string".to_owned()))
        })
        .collect::<LabResult<Vec<String>>>()?;
    Ok(hashes)
}

/// Read a transaction's confirmation count.
pub fn get_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let value = parse_cli_value(&raw)?;
    let confirmations = value
        .get("confirmations")
        .and_then(Value::as_i64)
        .ok_or_else(|| LabError::MissingField("confirmations"))?;
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
