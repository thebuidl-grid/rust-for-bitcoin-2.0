//! Lab 01 — build and verify a regtest network.

use serde_json::Value;

use crate::model::NetworkSnapshot;
use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::{LabError, LabResult};

/// Return the active chain reported by `getblockchaininfo`.
pub fn get_chain<C: RpcClient>(client: &C) -> LabResult<String> {
    let call = client.call(None, "getblockchaininfo", &[])?;
    let response = parse_cli_value(&call)?;
    required_string(&response, "chain")
}

/// Return the current height using the node-wide `getblockcount` RPC.
pub fn get_block_height<C: RpcClient>(client: &C) -> LabResult<u64> {
    let call = client.call(None, "getblockcount", &[])?;
    let response = parse_cli_value(&call)?;
    response
        .as_u64()
        .ok_or(LabError::Parse("expected u64".to_string()))
}

/// Return the node's current best-block hash.
pub fn get_best_block_hash<C: RpcClient>(client: &C) -> LabResult<String> {
    let call = client.call(None, "getbestblockhash", &[])?;
    let response = parse_cli_value(&call)?;
    match response {
        Value::String(s) => Ok(s),
        _ => Err(LabError::Parse("expected string".to_string())),
    }
}

/// Collect the chain, height, and best-block hash into one snapshot.
pub fn inspect_network<C: RpcClient>(client: &C) -> LabResult<NetworkSnapshot> {
    let chain = get_chain(client)?;
    let block_height = get_block_height(client)?;
    let best_block_hash = get_best_block_hash(client)?;

    if chain != "regtest" {
        return Err(LabError::Rpc(format!("expected regtest, found {}", chain)));
    }
    Ok(NetworkSnapshot {
        chain,
        block_height,
        best_block_hash,
    })
}
