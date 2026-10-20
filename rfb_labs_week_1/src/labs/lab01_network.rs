//! Lab 01 — build and verify a regtest network.

use crate::model::NetworkSnapshot;
use crate::rpc::RpcClient;
use crate::LabResult;

use crate::rpc::{parse_cli_value, required_string};
use crate::LabError;

/// Return the active chain reported by `getblockchaininfo`.
pub fn get_chain<C: RpcClient>(client: &C) -> LabResult<String> {
    let raw = client.call(None, "getblockchaininfo", &[])?;
    let json = parse_cli_value(&raw)?;
    required_string(&json, "chain")
}

/// Return the current height using the node-wide `getblockcount` RPC.
pub fn get_block_height<C: RpcClient>(client: &C) -> LabResult<u64> {
    let raw = client.call(None, "getblockcount", &[])?;
    let json = parse_cli_value(&raw)?;
    json.as_u64()
        .ok_or_else(|| LabError::Parse("block height is not a valid number".to_owned()))
}

/// Return the node's current best-block hash.
pub fn get_best_block_hash<C: RpcClient>(client: &C) -> LabResult<String> {
    let raw = client.call(None, "getbestblockhash", &[])?;
    let json = parse_cli_value(&raw)?;
    json.as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| LabError::Parse("best block hash is not a string".to_owned()))
}

/// Collect the chain, height, and best-block hash into one snapshot.
pub fn inspect_network<C: RpcClient>(client: &C) -> LabResult<NetworkSnapshot> {
    let chain = get_chain(client)?;
    if chain != "regtest" {
        return Err(LabError::Rpc(format!("Expected regtest network, found: {chain}")));
    }
    let block_height = get_block_height(client)?;
    let best_block_hash = get_best_block_hash(client)?;

    Ok(NetworkSnapshot {
        chain,
        block_height,
        best_block_hash,
    })
}
