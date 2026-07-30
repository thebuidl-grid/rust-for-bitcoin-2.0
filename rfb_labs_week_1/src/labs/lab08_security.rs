//! Lab 08 — inspect proof-linked headers and confirmation depth.

use crate::model::{BlockHeaderEvidence, SecurityReport};
use crate::rpc::RpcClient;
use crate::LabError;
use crate::LabResult;

/// Decode a block header into the fields used by the lab.
pub fn get_block_header<C: RpcClient>(
    client: &C,
    block_hash: &str,
) -> LabResult<BlockHeaderEvidence> {
    // TODO: call getblockheader with verbose output and decode all model fields.
    let raw = client.call(None, "getblockheader", &[block_hash.to_owned()])?;
    let value = crate::rpc::parse_cli_value(&raw)?;

    Ok(BlockHeaderEvidence {
        hash: crate::rpc::required_string(&value, "hash")?,
        height: crate::rpc::required_u64(&value, "height")?,
        previous_block_hash: value
            .get("previousblockhash")
            .and_then(|v| v.as_str())
            .map(ToOwned::to_owned),
        merkle_root: crate::rpc::required_string(&value, "merkleroot")?,
        nonce: crate::rpc::required_u64(&value, "nonce")?,
        difficulty: crate::rpc::required_f64(&value, "difficulty")?,
        bits: crate::rpc::required_string(&value, "bits")?,
        confirmations: value
            .get("confirmations")
            .and_then(|v| v.as_i64())
            .ok_or(LabError::MissingField("confirmations"))?,
        chainwork: crate::rpc::required_string(&value, "chainwork")?,
    })
}

/// Mine an exact number of additional blocks and return their hashes.
pub fn mine_additional_blocks<C: RpcClient>(
    client: &C,
    miner_address: &str,
    count: u64,
) -> LabResult<Vec<String>> {
    // TODO: call generatetoaddress.
    crate::labs::lab03_maturity::mine_blocks(client, miner_address, count)
}

/// Read a transaction's confirmation count.
pub fn get_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    // TODO: call gettransaction and return confirmations.
    crate::labs::lab07_confirm::transaction_confirmations(client, wallet_name, txid)
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
