//! Lab 08 — inspect proof-linked headers and confirmation depth.

use crate::model::{BlockHeaderEvidence, SecurityReport};
use crate::rpc::RpcClient;
use crate::{LabError, LabResult};
use serde_json::Value;

/// Decode a block header into the fields used by the lab.
pub fn get_block_header<C: RpcClient>(
    client: &C,
    block_hash: &str,
) -> LabResult<BlockHeaderEvidence> {
    let raw = client.call(None, "getblockheader", &[block_hash.to_string()])?;
    let value: Value = serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    Ok(BlockHeaderEvidence {
        hash: value["hash"]
            .as_str()
            .ok_or(LabError::MissingField("hash"))?
            .to_string(),
        height: value["height"]
            .as_u64()
            .ok_or(LabError::MissingField("height"))?,
        previous_block_hash: value["previousblockhash"].as_str().map(ToOwned::to_owned),
        merkle_root: value["merkleroot"]
            .as_str()
            .ok_or(LabError::MissingField("merkleroot"))?
            .to_string(),
        nonce: value["nonce"]
            .as_u64()
            .ok_or(LabError::MissingField("nonce"))?,
        difficulty: value["difficulty"]
            .as_f64()
            .ok_or(LabError::MissingField("difficulty"))?,
        bits: value["bits"]
            .as_str()
            .ok_or(LabError::MissingField("bits"))?
            .to_string(),
        confirmations: value["confirmations"]
            .as_i64()
            .ok_or(LabError::MissingField("confirmations"))?,
        chainwork: value["chainwork"]
            .as_str()
            .ok_or(LabError::MissingField("chainwork"))?
            .to_string(),
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
    let value: Value = serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    value
        .as_array()
        .ok_or(LabError::MissingField("generatetoaddress"))?
        .iter()
        .map(|hash| hash.as_str().map(ToOwned::to_owned))
        .collect::<Option<Vec<String>>>()
        .ok_or(LabError::MissingField("generatetoaddress"))
}

/// Read a transaction's confirmation count.
pub fn get_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let value: Value = serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    value["confirmations"]
        .as_i64()
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
