//! Lab 01 — build and verify a regtest network.

use crate::model::NetworkSnapshot;
use crate::rpc::{parse_cli_value, RpcClient};
use crate::LabResult;

/// Return the active chain reported by `getblockchaininfo`.
pub fn get_chain<C: RpcClient>(client: &C) -> LabResult<String> {
    let raw = client.call(None, "getblockchaininfo", &[])?;
    let value = parse_cli_value(&raw)?;

    value
        .get("chain")
        .and_then(|v| v.as_str())
        .map(ToOwned::to_owned)
        .ok_or(crate::LabError::MissingField("chain"))
}

/// Return the current height using the node-wide `getblockcount` RPC.
pub fn get_block_height<C: RpcClient>(client: &C) -> LabResult<u64> {
    let raw = client.call(None, "getblockcount", &[])?;
    let value = parse_cli_value(&raw)?;

    value.as_u64().ok_or(crate::LabError::Parse(
        "block height is not a number".to_string(),
    ))
}

/// Return the node's current best-block hash.
pub fn get_best_block_hash<C: RpcClient>(client: &C) -> LabResult<String> {
    let raw = client.call(None, "getbestblockhash", &[])?;
    let value = parse_cli_value(&raw)?;

    value
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or(crate::LabError::Parse(
            "best block hash is not a string".to_string(),
        ))
}

/// Collect the chain, height, and best-block hash into one snapshot.
pub fn inspect_network<C: RpcClient>(client: &C) -> LabResult<NetworkSnapshot> {
    let chain = get_chain(client)?;

    if chain != "regtest" {
        return Err(crate::LabError::Parse(
            "expected regtest network".to_string(),
        ));
    }

    let block_height = get_block_height(client)?;
    let best_block_hash = get_best_block_hash(client)?;

    Ok(NetworkSnapshot {
        chain,
        block_height,
        best_block_hash,
    })
}
