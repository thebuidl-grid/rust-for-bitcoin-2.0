//! Lab 08 — inspect proof-linked headers and confirmation depth.

use serde_json::Value;

use crate::model::{BlockHeaderEvidence, SecurityReport};
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

/// Decode a block header into the fields used by the lab.
pub fn get_block_header<C: RpcClient>(
    client: &C,
    block_hash: &str,
) -> LabResult<BlockHeaderEvidence> {
    let raw = client.call(None, "getblockheader", &[block_hash.to_string()])?;
    let value = parse_cli_value(&raw)?;

    Ok(BlockHeaderEvidence {
        hash: value
            .get("hash")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .ok_or(LabError::MissingField("hash"))?,
        height: value
            .get("height")
            .and_then(Value::as_u64)
            .ok_or(LabError::MissingField("height"))?,
        previous_block_hash: value
            .get("previousblockhash")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        merkle_root: value
            .get("merkleroot")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .ok_or(LabError::MissingField("merkleroot"))?,
        nonce: value
            .get("nonce")
            .and_then(Value::as_u64)
            .ok_or(LabError::MissingField("nonce"))?,
        difficulty: value
            .get("difficulty")
            .and_then(Value::as_f64)
            .ok_or(LabError::MissingField("difficulty"))?,
        bits: value
            .get("bits")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .ok_or(LabError::MissingField("bits"))?,
        confirmations: value
            .get("confirmations")
            .and_then(Value::as_i64)
            .ok_or(LabError::MissingField("confirmations"))?,
        chainwork: value
            .get("chainwork")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .ok_or(LabError::MissingField("chainwork"))?,
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
    let value = parse_cli_value(&raw)?;

    match value {
        Value::Array(items) => items
            .into_iter()
            .map(|item| {
                item.as_str()
                    .map(ToOwned::to_owned)
                    .ok_or(LabError::Parse("expected string block hash".to_owned()))
            })
            .collect(),
        other => Err(LabError::Parse(format!("expected array, got {other}"))),
    }
}

/// Read a transaction's confirmation count.
pub fn get_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let value = parse_cli_value(&raw)?;

    value
        .get("confirmations")
        .and_then(Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))
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
