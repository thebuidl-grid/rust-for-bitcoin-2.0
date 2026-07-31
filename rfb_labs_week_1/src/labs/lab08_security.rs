//! Lab 08 — inspect proof-linked headers and confirmation depth.

use crate::labs::lab03_maturity::mine_blocks;
use crate::labs::lab07_confirm::transaction_confirmations;
use crate::model::{BlockHeaderEvidence, SecurityReport};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

/// Blocks mined on top of the confirming block during this lab.
const ADDITIONAL_CONFIRMATIONS: u64 = 5;

/// Decode a block header into the fields used by the lab.
pub fn get_block_header<C: RpcClient>(
    client: &C,
    block_hash: &str,
) -> LabResult<BlockHeaderEvidence> {
    // `getblockheader` is verbose by default, so no verbosity argument is needed.
    let raw = client.call(None, "getblockheader", &[block_hash.to_owned()])?;
    let value = parse_cli_value(&raw)?;

    Ok(BlockHeaderEvidence {
        hash: required_string(&value, "hash")?,
        height: required_u64(&value, "height")?,
        // The genesis block has no parent.
        previous_block_hash: value
            .get("previousblockhash")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        merkle_root: required_string(&value, "merkleroot")?,
        nonce: required_u64(&value, "nonce")?,
        difficulty: required_f64(&value, "difficulty")?,
        bits: required_string(&value, "bits")?,
        confirmations: value
            .get("confirmations")
            .and_then(Value::as_i64)
            .ok_or(LabError::MissingField("confirmations"))?,
        chainwork: required_string(&value, "chainwork")?,
    })
}

/// Mine an exact number of additional blocks and return their hashes.
pub fn mine_additional_blocks<C: RpcClient>(
    client: &C,
    miner_address: &str,
    count: u64,
) -> LabResult<Vec<String>> {
    mine_blocks(client, miner_address, count)
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

    // Each new block buries the payment one level deeper without altering it.
    mine_additional_blocks(client, miner_address, ADDITIONAL_CONFIRMATIONS)?;

    Ok(SecurityReport {
        header,
        confirmations_before,
        confirmations_after: get_confirmations(client, wallet_name, txid)?,
    })
}
