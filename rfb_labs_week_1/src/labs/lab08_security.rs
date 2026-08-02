//! Lab 08 — inspect proof-linked headers and confirmation depth.

use crate::labs::lab03_maturity::mine_blocks;
use crate::model::{BlockHeaderEvidence, SecurityReport};
use crate::rpc::{parse_cli_value, required_f64, required_string, RpcClient};
use crate::{LabError, LabResult};

/// Decode a block header into the fields used by the lab.
pub fn get_block_header<C: RpcClient>(
    client: &C,
    block_hash: &str,
) -> LabResult<BlockHeaderEvidence> {
    // 1. Call `getblockheader` (defaults to verbose JSON)
    let raw = client.call(None, "getblockheader", &[block_hash.into()])?;
    let value = parse_cli_value(&raw)?;

    // 2. Extract header fields
    let hash = required_string(&value, "hash")?;
    let height = value["height"]
        .as_u64()
        .ok_or_else(|| LabError::Parse("getblockheader missing u64 'height'".to_owned()))?;

    let confirmations = value["confirmations"]
        .as_i64()
        .ok_or_else(|| LabError::Parse("getblockheader missing i64 'confirmations'".to_owned()))?;

    let merkle_root = required_string(&value, "merkleroot")?;

    // `previousblockhash` is optional (absent only for genesis block)
    let previous_block_hash = value["previousblockhash"].as_str().map(ToOwned::to_owned);

    let bits = required_string(&value, "bits")?;

    let nonce = value["nonce"]
        .as_u64()
        .ok_or_else(|| LabError::Parse("getblockheader missing u64 'nonce'".to_owned()))?;

    let difficulty = required_f64(&value, "difficulty")?;
    let chainwork = required_string(&value, "chainwork")?;

    Ok(BlockHeaderEvidence {
        hash,
        height,
        confirmations,
        merkle_root,
        previous_block_hash,
        bits,
        nonce,
        difficulty,
        chainwork,
    })
}

/// Mine an exact number of additional blocks and return their hashes.
pub fn mine_additional_blocks<C: RpcClient>(
    client: &C,
    miner_address: &str,
    count: u64,
) -> LabResult<Vec<String>> {
    // Re-use mine_blocks helper from lab_03
    mine_blocks(client, miner_address, count)
}

/// Read a transaction's confirmation count.
pub fn get_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.into()])?;
    let value = parse_cli_value(&raw)?;

    value["confirmations"]
        .as_i64()
        .ok_or_else(|| LabError::Parse("gettransaction missing integer 'confirmations'".to_owned()))
}

/// Record the block header and prove one confirmation becomes six after five blocks.
pub fn build_security_report<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
    block_hash: &str,
    miner_address: &str,
) -> LabResult<SecurityReport> {
    // 1. Inspect initial block header and initial transaction confirmations (should be 1)
    let header = get_block_header(client, block_hash)?;
    let confirmations_before = get_confirmations(client, wallet_name, txid)?;

    // 2. Mine 5 additional blocks on top of the tip
    let _ = mine_additional_blocks(client, miner_address, 5)?;

    // 3. Re-read the transaction status (should now show 6 confirmations)
    let confirmations_after = get_confirmations(client, wallet_name, txid)?;

    Ok(SecurityReport {
        header,
        confirmations_before,
        confirmations_after,
    })
}