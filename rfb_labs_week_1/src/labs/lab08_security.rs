//! Lab 08 — inspect proof-linked headers and confirmation depth.

use crate::model::{BlockHeaderEvidence, SecurityReport};
use crate::rpc::{parse_cli_value, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};

/// Decode a block header into the fields used by the lab.
pub fn get_block_header<C: RpcClient>(
    client: &C,
    block_hash: &str,
) -> LabResult<BlockHeaderEvidence> {
    let call = client.call(None, "getblockheader", &[block_hash.to_string()])?;
    let val = parse_cli_value(&call)?;

    let hash = required_string(&val, "hash")?;
    let height = required_u64(&val, "height")?;
    let previous_block_hash = val
        .get("previousblockhash")
        .and_then(|v| v.as_str())
        .map(String::from);
    let confirmations = val
        .get("confirmations")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| LabError::MissingField("confirmations"))?;
    let difficulty = val
        .get("difficulty")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| LabError::MissingField("difficulty"))?;
    let nonce = required_u64(&val, "nonce")?;
    let bits = required_string(&val, "bits")?;
    let chainwork = required_string(&val, "chainwork")?;
    let merkle_root = required_string(&val, "merkleroot")?;

    Ok(BlockHeaderEvidence {
        hash,
        height,
        previous_block_hash,
        confirmations,
        difficulty,
        nonce,
        bits,
        chainwork,
        merkle_root,
    })
}

/// Mine an exact number of additional blocks and return their hashes.
pub fn mine_additional_blocks<C: RpcClient>(
    client: &C,
    miner_address: &str,
    count: u64,
) -> LabResult<Vec<String>> {
    let call = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), miner_address.to_string()],
    )?;
    let val = parse_cli_value(&call)?;

    serde_json::from_value::<Vec<String>>(val).map_err(Into::into)
}

/// Read a transaction's confirmation count.
pub fn get_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let call = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let val = parse_cli_value(&call)?;

    val.get("confirmations")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| LabError::MissingField("confirmations"))
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
