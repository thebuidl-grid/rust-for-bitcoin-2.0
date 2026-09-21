//! Lab 01 — build and verify a regtest network.

use crate::model::NetworkSnapshot;
use crate::rpc::RpcClient;
use crate::LabResult;
use serde_json::Value;
use crate::error::LabError;
use crate::rpc::parse_cli_value;

/// Return the active chain reported by `getblockchaininfo`.
pub fn get_chain<C: RpcClient>(client: &C) -> LabResult<String> {
    // TODO: call getblockchaininfo and return its `chain` field.
     let info = client.call(None, "getblockchaininfo", &[])?;
     let value = parse_cli_value(&info)?;
        crate::rpc::required_string(&value, "chain")
}

/// Return the current height using the node-wide `getblockcount` RPC.
pub fn get_block_height<C: RpcClient>(client: &C) -> LabResult<u64> {
    let raw = client.call(None, "getblockcount", &[])?;
    let val = parse_cli_value(&raw)?;
    val.as_u64()
        .ok_or_else(|| LabError::Parse("Invalid block count".into()))
}

/// Return the node's current best-block hash.
pub fn get_best_block_hash<C: RpcClient>(client: &C) -> LabResult<String> {
    let raw = client.call(None, "getbestblockhash", &[])?;
    let val = parse_cli_value(&raw)?;
    val.as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| LabError::Parse("Invalid best block hash".into()))
}

/// Collect the chain, height, and best-block hash into one snapshot.
pub fn inspect_network<C: RpcClient>(client: &C) -> LabResult<NetworkSnapshot> {
    // TODO: compose the three functions above and reject non-regtest networks.
     let chain = get_chain(client)?;
    if chain != "regtest" {
        return Err(LabError::Rpc(format!("Expected regtest, got '{chain}'")));
    }

    let block_height = get_block_height(client)?;
    let best_block_hash = get_best_block_hash(client)?;

    Ok(NetworkSnapshot {
        chain,
        block_height,
        best_block_hash,
    })
}
