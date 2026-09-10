//! Lab 08 — inspect proof-linked headers and confirmation depth.

use crate::labs::lab03_maturity::mine_blocks;
use crate::labs::lab07_confirm::transaction_confirmations;
use crate::model::{BlockHeaderEvidence, SecurityReport};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};

/// Decode a block header into the fields used by the lab.
pub fn get_block_header<C: RpcClient>(
    client: &C,
    block_hash: &str,
) -> LabResult<BlockHeaderEvidence> {
    let raw = client.call(None, "getblockheader", &[block_hash.to_string()])?;
    let value = parse_cli_value(&raw)?;
    let hash = required_string(&value, "hash")?;
    let height = required_u64(&value, "height")?;
    let previous_block_hash = value
        .get("previousblockhash")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned);
    let merkle_root = required_string(&value, "merkleroot")?;
    let nonce = required_u64(&value, "nonce")?;
    let difficulty = required_f64(&value, "difficulty")?;
    let bits = required_string(&value, "bits")?;
    let confirmations_u64 = value
        .get("confirmations")
        .and_then(serde_json::Value::as_u64)
        .ok_or(LabError::MissingField("confirmations"))?;
    let confirmations: i64 = confirmations_u64 as i64;
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
    let hashes = mine_blocks(client, &miner_address, count)?;
    Ok(hashes)
}

/// Read a transaction's confirmation count.
pub fn get_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    transaction_confirmations(client, wallet_name, txid)
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
