//! Lab 08 — inspect proof-linked headers and confirmation depth.

use crate::model::{BlockHeaderEvidence, SecurityReport};
use crate::rpc::RpcClient;
use crate::{LabResult, LabError};
use serde::Deserialize;

#[derive(Deserialize)]
struct RawBlockHeader {
    hash: String,
    height: u64,
    #[serde(rename = "previousblockhash")]
    previous_block_hash: Option<String>,
    #[serde(rename = "merkleroot")]
    merkle_root: String,
    nonce: u64,
    difficulty: f64,
    bits: String,
    confirmations: i64,
    chainwork: String,
}

/// Decode a block header into the fields used by the lab.
pub fn get_block_header<C: RpcClient>(
    client: &C,
    block_hash: &str,
) -> LabResult<BlockHeaderEvidence> {
    // TODO: call getblockheader with verbose output and decode all model fields.
    // todo!("Lab 08: inspect a block header")
    let raw = client.call(None, "getblockheader", &[block_hash.to_string()])?;
    let parsed: RawBlockHeader =
        serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    Ok(BlockHeaderEvidence {
        hash: parsed.hash,
        height: parsed.height,
        previous_block_hash: parsed.previous_block_hash,
        merkle_root: parsed.merkle_root,
        nonce: parsed.nonce,
        difficulty: parsed.difficulty,
        bits: parsed.bits,
        confirmations: parsed.confirmations,
        chainwork: parsed.chainwork,
    })
}

/// Mine an exact number of additional blocks and return their hashes.
pub fn mine_additional_blocks<C: RpcClient>(
    client: &C,
    miner_address: &str,
    count: u64,
) -> LabResult<Vec<String>> {
    // TODO: call generatetoaddress.
    // todo!("Lab 08: mine additional confirmations")
    let raw = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), miner_address.to_string()],
    )?;

    serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))
}

#[derive(Deserialize)]
struct RawTransaction {
    confirmations: i64,
}

/// Read a transaction's confirmation count.
pub fn get_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    // TODO: call gettransaction and return confirmations.
    // todo!("Lab 08: read confirmation depth")
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let parsed: RawTransaction =
        serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    Ok(parsed.confirmations)
}

/// Record the block header and prove one confirmation becomes six after five blocks.
pub fn build_security_report<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
    block_hash: &str,
    miner_address: &str,
) -> LabResult<SecurityReport> {
    // TODO: read header and initial confirmations, mine five blocks, then read again.
    // todo!("Lab 08: build proof-of-work and confirmation evidence")
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
