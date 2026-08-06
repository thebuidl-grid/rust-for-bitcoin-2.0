//! Lab 01 — build and verify a regtest network.

use crate::model::NetworkSnapshot;
use crate::rpc::{parse_cli_value, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};

/// Return the active chain reported by `getblockchaininfo`.
pub fn get_chain<C: RpcClient>(client: &C) -> LabResult<String> {
    let response = client.call(None, "getblockchaininfo", &[])?;

    let parsed_response = parse_cli_value(&response)?;

    required_string(&parsed_response, "chain")
}

/// Return the current height using the node-wide `getblockcount` RPC.
pub fn get_block_height<C: RpcClient>(client: &C) -> LabResult<u64> {
    let response = client.call(None, "getblockcount", &[])?;

    let parsed_response = parse_cli_value(&response)?;

    parsed_response
        .as_u64()
        .ok_or_else(|| LabError::Parse("Expected an unsigned integer".to_string()))
}

/// Return the node's current best-block hash.
pub fn get_best_block_hash<C: RpcClient>(client: &C) -> LabResult<String> {
    let response = client.call(None, "getbestblockhash", &[])?;

    Ok(response)
}

/// Collect the chain, height, and best-block hash into one snapshot.
pub fn inspect_network<C: RpcClient>(client: &C) -> LabResult<NetworkSnapshot> {
    let chain = get_chain(client)?;

    if chain != "regtest" {
        return Err(LabError::Parse("Invalid chain".to_owned()));
    }

    let block_height = get_block_height(client)?;
    let best_block_hash = get_best_block_hash(client)?;

    let snapshot = NetworkSnapshot {
        chain,
        block_height,
        best_block_hash,
    };

    Ok(snapshot)
}
