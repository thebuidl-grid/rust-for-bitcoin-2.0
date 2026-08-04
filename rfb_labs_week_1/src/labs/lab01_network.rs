//! Lab 01 — build and verify a regtest network.

use crate::model::NetworkSnapshot;
use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::{LabError, LabResult};

/// The only chain these labs are allowed to run against.
const EXPECTED_CHAIN: &str = "regtest";

/// Return the active chain reported by `getblockchaininfo`.
pub fn get_chain<C: RpcClient>(client: &C) -> LabResult<String> {
    let raw = client.call(None, "getblockchaininfo", &[])?;
    required_string(&parse_cli_value(&raw)?, "chain")
}

/// Return the current height using the node-wide `getblockcount` RPC.
pub fn get_block_height<C: RpcClient>(client: &C) -> LabResult<u64> {
    let raw = client.call(None, "getblockcount", &[])?;
    parse_cli_value(&raw)?
        .as_u64()
        .ok_or_else(|| LabError::Parse(format!("getblockcount returned `{raw}`")))
}

/// Return the node's current best-block hash.
pub fn get_best_block_hash<C: RpcClient>(client: &C) -> LabResult<String> {
    let raw = client.call(None, "getbestblockhash", &[])?;
    parse_cli_value(&raw)?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| LabError::Parse(format!("getbestblockhash returned `{raw}`")))
}

/// Collect the chain, height, and best-block hash into one snapshot.
pub fn inspect_network<C: RpcClient>(client: &C) -> LabResult<NetworkSnapshot> {
    let chain = get_chain(client)?;

    // Every later lab mines blocks and spends coins, so stop the moment the node turns
    // out to be anything other than a throwaway regtest chain.
    if chain != EXPECTED_CHAIN {
        return Err(LabError::Parse(format!(
            "expected the `{EXPECTED_CHAIN}` chain but the node reports `{chain}`"
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
