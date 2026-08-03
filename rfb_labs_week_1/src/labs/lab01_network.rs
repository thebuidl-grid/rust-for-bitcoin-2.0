//! Lab 01 — build and verify a regtest network.

use crate::model::NetworkSnapshot;
use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::{LabError, LabResult};

/// Return the active chain reported by `getblockchaininfo`.
pub fn get_chain<C: RpcClient>(client: &C) -> LabResult<String> {
    let raw = client.call(None, "getblockchaininfo", &[])?;
    required_string(&parse_cli_value(&raw)?, "chain")
}

/// Return the current height using the node-wide `getblockcount` RPC.
pub fn get_block_height<C: RpcClient>(client: &C) -> LabResult<u64> {
    let raw = client.call(None, "getblockcount", &[])?;
    raw.trim()
        .parse()
        .map_err(|_| LabError::Parse(format!("getblockcount returned `{}`", raw.trim())))
}

/// Return the node's current best-block hash.
pub fn get_best_block_hash<C: RpcClient>(client: &C) -> LabResult<String> {
    // `bitcoin-cli` prints hashes as bare text rather than a quoted JSON string.
    let raw = client.call(None, "getbestblockhash", &[])?;
    Ok(raw.trim().to_owned())
}

/// Collect the chain, height, and best-block hash into one snapshot.
pub fn inspect_network<C: RpcClient>(client: &C) -> LabResult<NetworkSnapshot> {
    let chain = get_chain(client)?;
    if chain != "regtest" {
        return Err(LabError::Rpc(format!(
            "expected the regtest chain, found `{chain}`"
        )));
    }

    Ok(NetworkSnapshot {
        chain,
        block_height: get_block_height(client)?,
        best_block_hash: get_best_block_hash(client)?,
    })
}
