//! Lab 01 — build and verify a regtest network.

use crate::model::NetworkSnapshot;
use crate::rpc::{parse_cli_value, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};

/// Return the active chain reported by `getblockchaininfo`.
pub fn get_chain<C: RpcClient>(client: &C) -> LabResult<String> {
    // TODO: call getblockchaininfo and return its `chain` field.
    let call = client.call(None, "getblockchaininfo", &[])?;
    let response = parse_cli_value(&call)?;
    required_string(&response, "chain")
}

/// Return the current height using the node-wide `getblockcount` RPC.
pub fn get_block_height<C: RpcClient>(client: &C) -> LabResult<u64> {
    // TODO: call getblockcount and parse the numeric response.

    let call = client.call(None, "getblockcount", &[])?;
    let response = parse_cli_value(&call)?;

    response.as_u64().ok_or(LabError::Parse(
        "getblockcount response is not a u64".to_string(),
    ))
}

/// Return the node's current best-block hash.
pub fn get_best_block_hash<C: RpcClient>(client: &C) -> LabResult<String> {
    let call = client.call(None, "getbestblockhash", &[])?;
    let response = parse_cli_value(&call)?;

    response
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| LabError::Parse("getbestblockhash response is not a string".to_string()))
}

pub fn inspect_network<C: RpcClient>(client: &C) -> LabResult<NetworkSnapshot> {
    let chain = get_chain(client)?;
    if chain != "regtest" {
        return Err(LabError::Parse(format!(
            "expected regtest chain, got {chain}"
        )));
    }

    let block_height = get_block_height(client)?;
    let best_block_hash = get_best_block_hash(client)?;

    Ok(NetworkSnapshot {
        chain,
        block_height,
        best_block_hash,
    })
}
