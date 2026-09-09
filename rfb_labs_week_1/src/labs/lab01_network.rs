//! Lab 01 — build and verify a regtest network.

use crate::model::NetworkSnapshot;
use crate::rpc::RpcClient;
use crate::LabError;
use crate::LabResult;

/// Return the active chain reported by `getblockchaininfo`.
pub fn get_chain<C: RpcClient>(client: &C) -> LabResult<String> {
    // TODO: call getblockchaininfo and return its `chain` field.
    let raw = client.call(None, "getblockchaininfo", &[])?;
    let value = crate::rpc::parse_cli_value(&raw)?;
    crate::rpc::required_string(&value, "chain")
    //todo!("Lab 01: read the active chain")
}

/// Return the current height using the node-wide `getblockcount` RPC.
pub fn get_block_height<C: RpcClient>(client: &C) -> LabResult<u64> {
    let raw = client.call(None, "getblockcount", &[])?;
    let value = crate::rpc::parse_cli_value(&raw)?;
    value
        .as_u64()
        .ok_or_else(|| LabError::Parse(format!("expected a number, got: {value}")))
    // TODO: call getblockcount and parse the numeric response.
    // todo!("Lab 01: read the current block height")
}

/// Return the node's current best-block hash.
pub fn get_best_block_hash<C: RpcClient>(client: &C) -> LabResult<String> {
    let raw = client.call(None, "getbestblockhash", &[])?;
    let value = crate::rpc::parse_cli_value(&raw)?;
    value
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| LabError::Parse(format!("expected a string, got: {value}")))
    // TODO: call getbestblockhash.
    //todo!("Lab 01: read the best-block hash")
}

/// Collect the chain, height, and best-block hash into one snapshot.
pub fn inspect_network<C: RpcClient>(client: &C) -> LabResult<NetworkSnapshot> {
    // TODO: compose the three functions above and reject non-regtest networks.
    //todo!("Lab 01: build a verified regtest snapshot")
    let chain = get_chain(client)?;
    let block_height = get_block_height(client)?;
    let best_block_hash = get_best_block_hash(client)?;

    if chain != "regtest" {
        return Err(LabError::Rpc(format!(
            "expected regtest network, got: {chain}"
        )));
    }

    Ok(NetworkSnapshot {
        chain,
        block_height,
        best_block_hash,
    })
}
