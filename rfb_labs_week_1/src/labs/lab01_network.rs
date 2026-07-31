//! Lab 01 — build and verify a regtest network.

use crate::model::NetworkSnapshot;
use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::{LabError, LabResult};

/// Return the active chain reported by `getblockchaininfo`.
pub fn get_chain<C: RpcClient>(client: &C) -> LabResult<String> {
    // TODO: call getblockchaininfo and return its `chain` field.
    let response = client.call(None, "getblockchaininfo", &[])?;
    let value = parse_cli_value(&response)?;
    required_string(&value, "chain")
}

/// Return the current height using the node-wide `getblockcount` RPC.
pub fn get_block_height<C: RpcClient>(client: &C) -> LabResult<u64> {
    // TODO: call getblockcount and parse the numeric response.
    let response = client.call(None, "getblockcount", &[])?;
    let value = parse_cli_value(&response)?;

    value
        .as_u64()
        .ok_or(LabError::Parse("expected block height".to_string()))
}

/// Return the node's current best-block hash.
pub fn get_best_block_hash<C: RpcClient>(client: &C) -> LabResult<String> {
    // TODO: call getbestblockhash.
     client.call(None, "getbestblockhash", &[])
}

/// Collect the chain, height, and best-block hash into one snapshot.
pub fn inspect_network<C: RpcClient>(client: &C) -> LabResult<NetworkSnapshot> {
    // TODO: compose the three functions above and reject non-regtest networks.
    let chain = get_chain(client)?;

    if chain != "regtest" {
        return Err(LabError::Parse(format!(
            "expected regtest network, found {chain}"
        )));
    }

    Ok(NetworkSnapshot {
        chain,
        block_height: get_block_height(client)?,
        best_block_hash: get_best_block_hash(client)?,
    })
}
