//! Lab 01 — build and verify a regtest network.

use crate::model::NetworkSnapshot;
use crate::rpc::RpcClient;
use crate::{LabResult, LabError};
use serde_json::Value;

/// Return the active chain reported by `getblockchaininfo`.
pub fn get_chain<C: RpcClient>(client: &C) -> LabResult<String> {
    // TODO: call getblockchaininfo and return its `chain` field.
     let raw = client.call(None, "getblockchaininfo", &[])?;
    let info: Value = serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;
    // todo!("Lab 01: read the active chain");
    let chain = info["chain"]
        .as_str()
        .ok_or(LabError::MissingField("chain"))?
        .to_string();

    Ok(chain)

    
}

/// Return the current height using the node-wide `getblockcount` RPC.
pub fn get_block_height<C: RpcClient>(client: &C) -> LabResult<u64> {
    // TODO: call getblockcount and parse the numeric response.
    // todo!("Lab 01: read the current block height")
    let raw = client.call(None, "getblockcount", &[])?;
    raw.trim()
        .parse::<u64>()
        .map_err(|_| LabError::Parse(format!("getblockcount did not return a valid u64: '{}'", raw.trim())))
}

/// Return the node's current best-block hash.
pub fn get_best_block_hash<C: RpcClient>(client: &C) -> LabResult<String> {
    // TODO: call getbestblockhash.
    // todo!("Lab 01: read the best-block hash")
    let raw = client.call(None, "getbestblockhash", &[])?;
    Ok(raw.trim().trim_matches('"').to_string())
    
}

/// Collect the chain, height, and best-block hash into one snapshot.
pub fn inspect_network<C: RpcClient>(client: &C) -> LabResult<NetworkSnapshot> {
    // TODO: compose the three functions above and reject non-regtest networks.
    // todo!("Lab 01: build a verified regtest snapshot")
    let chain = get_chain(client)?;

    if chain != "regtest" {
        return Err(LabError::Rpc(format!(
            "expected regtest network, but node reports chain = '{}'",
            chain
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


