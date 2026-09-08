//! Lab 01 — build and verify a regtest network.

use crate::model::NetworkSnapshot;
use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::{LabError, LabResult};

/// Returning the active chain reported by `getblockchaininfo`.
pub fn get_chain<C: RpcClient>(client: &C) -> LabResult<String> {
    
    let raw_info = client.call(None, "getblockchaininfo", &[])?;
    let info = parse_cli_value(&raw_info)?;
    required_string(&info, "chain")
}

/// Returning the current height using the node-wide `getblockcount` RPC.
pub fn get_block_height<C: RpcClient>(client: &C) -> LabResult<u64> {
    
    let raw_info = client.call(None, "getblockcount", &[])?;
    let value = parse_cli_value(&raw_info)?;
    value.as_u64().ok_or(LabError::MissingField("blockcount"))
}

/// Return the node's current best-block hash.
pub fn get_best_block_hash<C: RpcClient>(client: &C) -> LabResult<String> {
    // TODO: call getbestblockhash.
    let raw_info = client.call(None, "getbestblockhash", &[])?;
    let value = parse_cli_value(&raw_info)?;
    value.as_str().map(ToOwned::to_owned).ok_or(LabError::MissingField("getblockhash"))
}

/// Collect the chain, height, and best-block hash into one snapshot.
pub fn inspect_network<C: RpcClient>(client: &C) -> LabResult<NetworkSnapshot> {
    // TODO: compose the three functions above and reject non-regtest networks.
    let chain = get_chain(client)?;
    if chain != "regtest"{
        return Err(LabError::Rpc(format!(
            "Expected regtest, node reports chain \"{chain}\""
        )));
    }
    let block_height = get_block_height(client)?;
    let best_block_hash = get_best_block_hash(client)?;

    Ok(NetworkSnapshot { chain, block_height, best_block_hash })
}
