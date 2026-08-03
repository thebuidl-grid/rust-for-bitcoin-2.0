//! Lab 01 — build and verify a regtest network.

use crate::model::NetworkSnapshot;
use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::{LabError, LabResult};

/// Return the active chain reported by `getblockchaininfo`.
pub fn get_chain<C: RpcClient>(client: &C) -> LabResult<String> {
    let raw = client.call(None, "getblockchaininfo", &[])?;
    let info = parse_cli_value(&raw)?;
    required_string(&info, "chain")
}

/// Return the current height using the node-wide `getblockcount` RPC.
pub fn get_block_height<C: RpcClient>(client: &C) -> LabResult<u64> {
    let raw = client.call(None, "getblockcount", &[])?;
    let count = parse_cli_value(&raw)?;
    count
        .as_u64()
        .ok_or_else(|| LabError::Parse(format!("getblockcount returned a non-number: {raw}")))
}

/// Return the node's current best-block hash.
pub fn get_best_block_hash<C: RpcClient>(client: &C) -> LabResult<String> {
    let raw = client.call(None, "getbestblockhash", &[])?;
    let hash = parse_cli_value(&raw)?;
    hash.as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| LabError::Parse(format!("getbestblockhash returned a non-string: {raw}")))
}

/// Collect the chain, height, and best-block hash into one snapshot.
pub fn inspect_network<C: RpcClient>(client: &C) -> LabResult<NetworkSnapshot> {
    let chain = get_chain(client)?;
    if chain != "regtest" {
        return Err(LabError::Rpc(format!(
            "refusing to continue: expected the regtest chain but this node reports `{chain}`"
        )));
    }

    Ok(NetworkSnapshot {
        chain,
        block_height: get_block_height(client)?,
        best_block_hash: get_best_block_hash(client)?,
    })
}
