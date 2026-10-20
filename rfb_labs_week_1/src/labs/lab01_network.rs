//! Lab 01 — build and verify a regtest network.

use crate::model::NetworkSnapshot;
use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::{LabError, LabResult};

/// Return the active chain reported by `getblockchaininfo`.
pub fn get_chain<C: RpcClient>(client: &C) -> LabResult<String> {
    let response = client.call(None, "getblockchaininfo", &[])?;
    let blockchain_info = parse_cli_value(&response)?;

    required_string(&blockchain_info, "chain")
}

/// Return the current height using the node-wide `getblockcount` RPC.
pub fn get_block_height<C: RpcClient>(client: &C) -> LabResult<u64> {
    let response = client.call(None, "getblockcount", &[])?;
    let height = response
        .parse::<u64>()
        .map_err(|error| LabError::Parse(error.to_string()))?;

    Ok(height)
}

/// Return the node's current best-block hash.
pub fn get_best_block_hash<C: RpcClient>(client: &C) -> LabResult<String> {
    let response = client.call(None, "getbestblockhash", &[])?;
    let hash = parse_cli_value(&response)?;
    let hash = hash
        .as_str()
        .ok_or_else(|| LabError::Parse("expected a block hash".to_string()))?;

    Ok(hash.to_owned())
}

/// Collect the chain, height, and best-block hash into one snapshot.
pub fn inspect_network<C: RpcClient>(client: &C) -> LabResult<NetworkSnapshot> {
    let chain = get_chain(client)?;
    if chain != "regtest" {
        return Err(LabError::Parse(format!(
            "expected regtest, but the node reported `{chain}`"
        )));
    }

    let height = get_block_height(client)?;
    let best_block_hash = get_best_block_hash(client)?;

    Ok(NetworkSnapshot {
        chain,
        block_height: height,
        best_block_hash,
    })
}
