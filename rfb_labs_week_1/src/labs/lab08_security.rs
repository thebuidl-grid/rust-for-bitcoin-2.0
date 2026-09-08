//! Lab 08 — inspect proof-linked headers and confirmation depth.

use crate::labs::lab01_network::get_block_height;
use crate::labs::lab03_maturity::mine_blocks;
use crate::labs::lab07_confirm::{mine_one_block, transaction_confirmations};
use crate::model::{BlockHeaderEvidence, SecurityReport};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::LabResult;
use serde_json::Value;

/// Decode a block header into the fields used by the lab.
pub fn get_block_header<C: RpcClient>(
    client: &C,
    block_hash: &str,
) -> LabResult<BlockHeaderEvidence> {
    // TODO: call getblockheader with verbose output and decode all model fields.
    // todo!("Lab 08: inspect a block header")
    let result = client.call(None, "getblockheader", &[block_hash.to_string()])?;

    let decoded = parse_cli_value(&result)?;

    Ok(BlockHeaderEvidence {
        hash: required_string(&decoded, "hash")?,
        height: required_u64(&decoded, "height")?,
        previous_block_hash: decoded
            .get("previousblockhash")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        merkle_root: required_string(&decoded, "merkleroot")?,
        nonce: required_u64(&decoded, "nonce")?,
        difficulty: required_f64(&decoded, "difficulty")?,
        bits: required_string(&decoded, "bits")?,
        confirmations: decoded["confirmations"].as_i64().unwrap_or_default(),
        chainwork: required_string(&decoded, "chainwork")?,
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
    mine_blocks(client, miner_address, count)
}

/// Read a transaction's confirmation count.
pub fn get_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    // TODO: call gettransaction and return confirmations.
    // todo!("Lab 08: read confirmation depth")
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
    // TODO: read header and initial confirmations, mine five blocks, then read again.
    // todo!("Lab 08: build proof-of-work and confirmation evidence")
    let header = get_block_header(client, block_hash)?;
    let initial_confirmations = get_confirmations(client, wallet_name, txid)?;
    mine_additional_blocks(client, miner_address, 5)?;
    let final_confirmations = get_confirmations(client, wallet_name, txid)?;

    Ok(SecurityReport {
        header,
        confirmations_before: initial_confirmations,
        confirmations_after: final_confirmations,
    })
}
