//! Lab 08 — inspect proof-linked headers and confirmation depth.

use crate::model::{BlockHeaderEvidence, SecurityReport};
use crate::rpc::RpcClient;
use crate::LabResult;

use crate::rpc::parse_cli_value;
use crate::LabError;
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

impl From<RawBlockHeader> for BlockHeaderEvidence {
    fn from(raw: RawBlockHeader) -> Self {
        BlockHeaderEvidence {
            hash: raw.hash,
            height: raw.height,
            previous_block_hash: raw.previous_block_hash,
            merkle_root: raw.merkle_root,
            nonce: raw.nonce,
            difficulty: raw.difficulty,
            bits: raw.bits,
            confirmations: raw.confirmations,
            chainwork: raw.chainwork,
        }
    }
}

/// Decode a block header into the fields used by the lab.
pub fn get_block_header<C: RpcClient>(
    client: &C,
    block_hash: &str,
) -> LabResult<BlockHeaderEvidence> {
    let raw = client.call(None, "getblockheader", &[block_hash.to_owned()])?;
    let json = parse_cli_value(&raw)?;
    let raw_header: RawBlockHeader = serde_json::from_value(json)?;
    Ok(BlockHeaderEvidence::from(raw_header))
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
    let json = parse_cli_value(&raw)?;
    let hashes: Vec<String> = serde_json::from_value(json)?;
    Ok(hashes)
}

/// Read a transaction's confirmation count.
pub fn get_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let json = parse_cli_value(&raw)?;
    let confirmations = json.get("confirmations")
        .and_then(|v| v.as_i64())
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
