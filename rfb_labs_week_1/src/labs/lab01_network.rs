//! Lab 01 — build and verify a regtest network.

use crate::model::NetworkSnapshot;
use crate::rpc::parse_cli_value;
use crate::rpc::required_string;
use crate::rpc::RpcClient;
use crate::{LabError, LabResult};

/// Return the active chain reported by `getblockchaininfo`.
pub fn get_chain<C: RpcClient>(client: &C) -> LabResult<String> {
    // TODO: call getblockchaininfo and return its `chain` field.
    let call = client.call(None, "getblockchaininfo", &[])?;
    let response = parse_cli_value(&call)?;
    let chain: String = required_string(&response, "chain")?;
    Ok(chain)
}

/// Return the current height using the node-wide `getblockcount` RPC.
pub fn get_block_height<C: RpcClient>(client: &C) -> LabResult<u64> {
    // TODO: call getblockcount and parse the numeric response.
    // todo!("Lab 01: read the current block height");
    let call = client.call(None, "getblockcount", &[])?;
    let response = parse_cli_value(&call)?;
    let value = response.as_u64();
    match value {
        Some(value) => Ok(value),
        _ => Err(LabError::Parse("expected u64".to_string())),
    }
    // match value = {
    //     Some(value) => Ok(value),
    //     _ => Err(LabError::Parse("expected u64").to_string()),
    // }
}

/// Return the node's current best-block hash.
pub fn get_best_block_hash<C: RpcClient>(client: &C) -> LabResult<String> {
    // TODO: call getbestblockhash.
    // todo!("Lab 01: read the best-block hash")
    let call = client.call(None, "getbestblockhash", &[])?;
    let response = parse_cli_value(&call)?;

    match response {
        serde_json::Value::String(hash) => Ok(hash),
        _ => Err(LabError::Parse("expected string".to_string())),
    }
}

/// Collect the chain, height, and best-block hash into one snapshot.
pub fn inspect_network<C: RpcClient>(client: &C) -> LabResult<NetworkSnapshot> {
    // TODO: compose the three functions above and reject non-regtest networks.
    // todo!("Lab 01: build a verified regtest snapshot")
    let chain = get_chain(client)?;
    let block_height = get_block_height(client)?;
    let best_block_hash = get_best_block_hash(client)?;

    Ok(NetworkSnapshot {
        chain,
        block_height,
        best_block_hash,
    })
}
