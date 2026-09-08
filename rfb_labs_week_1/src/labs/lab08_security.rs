//! Lab 08 — inspect proof-linked headers and confirmation depth.
use crate::labs::lab03_maturity::mine_blocks;
use crate::labs::lab07_confirm::transaction_confirmations;
use crate::model::{BlockHeaderEvidence, SecurityReport};
use crate::rpc::{parse_cli_value,required_string, required_f64, required_u64, RpcClient};
use crate::{LabError, LabResult};

/// Decode a block header into the fields used by the lab.
pub fn get_block_header<C: RpcClient>(
    client: &C,
    block_hash: &str,
) -> LabResult<BlockHeaderEvidence> {
    // TODO: call getblockheader with verbose output and decode all model fields.
     let raw_info = client.call(None, "getblockheader", &[block_hash.to_string()])?;
    let info = parse_cli_value(&raw_info)?;

    let confirmations = info
        .get("confirmations")
        .and_then(serde_json::Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))?;

    let previous_block_hash = info
        .get("previousblockhash")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned);

    Ok(BlockHeaderEvidence {
        hash: required_string(&info, "hash")?,
        height: required_u64(&info, "height")?,
        previous_block_hash,
        merkle_root: required_string(&info, "merkleroot")?,
        nonce: required_u64(&info, "nonce")?,
        difficulty: required_f64(&info, "difficulty")?,
        bits: required_string(&info, "bits")?,
        confirmations,
        chainwork: required_string(&info, "chainwork")?,
    })
}

/// Mine an exact number of additional blocks and return their hashes.
pub fn mine_additional_blocks<C: RpcClient>(
    client: &C,
    miner_address: &str,
    count: u64,
) -> LabResult<Vec<String>> {
    // TODO: call generatetoaddress.
    mine_blocks(client, miner_address, count)
}

/// Read a transaction's confirmation count.
pub fn get_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    // TODO: call gettransaction and return confirmations.
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
