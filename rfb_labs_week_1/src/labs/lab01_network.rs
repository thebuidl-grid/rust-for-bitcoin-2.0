//! Lab 01 — build and verify a regtest network.

use crate::model::NetworkSnapshot;
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

/// Return the active chain reported by `getblockchaininfo`.
pub fn get_chain<C: RpcClient>(client: &C) -> LabResult<String> {
    // TODO: call getblockchaininfo and return its `chain` field.
    let result = client.call(None, "getblockchaininfo", &[]);

    match result {
        Ok(res) => {
            let parsed = parse_cli_value(&res)?;
            let chain = &parsed["chain"];
            Ok(chain.as_str().unwrap().to_string())
        }
        Err(err) => Err(err),
    }
}

/// Return the current height using the node-wide `getblockcount` RPC.
pub fn get_block_height<C: RpcClient>(client: &C) -> LabResult<u64> {
    // TODO: call getblockcount and parse the numeric response.
    // todo!("Lab 01: read the current block height")
    let result = client.call(None, "getblockcount", &[]);

    match result {
        Ok(res) => {
            let parsed = parse_cli_value(&res)?;
            Ok(parsed.as_u64().unwrap())
        }
        Err(err) => Err(err),
    }
}

/// Return the node's current best-block hash.
pub fn get_best_block_hash<C: RpcClient>(client: &C) -> LabResult<String> {
    // TODO: call getbestblockhash.
    // todo!("Lab 01: read the best-block hash")
    let result = client.call(None, "getbestblockhash", &[]);

    match result {
        Ok(res) => {
            let parsed = parse_cli_value(&res)?;
            Ok(parsed.as_str().unwrap().to_string())
        }
        Err(err) => Err(err),
    }
}

/// Collect the chain, height, and best-block hash into one snapshot.
pub fn inspect_network<C: RpcClient>(client: &C) -> LabResult<NetworkSnapshot> {
    // TODO: compose the three functions above and reject non-regtest networks.
    // todo!("Lab 01: build a verified regtest snapshot")
    let chain = get_chain(client)?;
    let block_height = get_block_height(client)?;
    let best_block_hash = get_best_block_hash(client)?;

    let result = NetworkSnapshot {
        chain,
        block_height,
        best_block_hash,
    };

    Ok(result)
}
