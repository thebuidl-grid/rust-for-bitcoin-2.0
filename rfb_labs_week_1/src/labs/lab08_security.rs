//! Lab 08 — inspect proof-linked headers and confirmation depth.

use crate::model::{BlockHeaderEvidence, SecurityReport};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};

/// Decode a block header into the fields used by the lab.
pub fn get_block_header<C: RpcClient>(
    client: &C,
    block_hash: &str,
) -> LabResult<BlockHeaderEvidence> {
    let response = client.call(None, "getblockheader", &[block_hash.to_string()])?;
    let value = parse_cli_value(&response)?;

    Ok(BlockHeaderEvidence {
        hash: required_string(&value, "hash")?,
        height: required_u64(&value, "height")?,
        previous_block_hash: value["previousblockhash"].as_str().map(String::from),
        merkle_root: required_string(&value, "merkleroot")?,
        nonce: required_u64(&value, "nonce")?,
        difficulty: required_f64(&value, "difficulty")?,
        bits: required_string(&value, "bits")?,
        confirmations: required_u64(&value, "confirmations")? as i64,
        chainwork: required_string(&value, "chainwork")?,
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

    let value = parse_cli_value(&response)?;

    Ok(value
        .as_array()
        .ok_or(LabError::Parse("expected array".into()))?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect())
}

/// Read a transaction's confirmation count.
pub fn get_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let response = client.call(
        Some(wallet_name),
        "gettransaction",
        &[txid.to_string()],
    )?;

    let value = parse_cli_value(&response)?;

    Ok(required_u64(&value, "confirmations")? as i64)
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