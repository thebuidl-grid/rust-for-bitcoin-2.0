//! Lab 01 — build and verify a regtest network.

use crate::model::NetworkSnapshot;
use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::{LabError, LabResult};

pub fn get_chain<C: RpcClient>(client: &C) -> LabResult<String> {
    let raw = client.call(None, "getblockchaininfo", &[])?;
    let value = parse_cli_value(&raw)?;
    required_string(&value, "chain")
}

pub fn get_block_height<C: RpcClient>(client: &C) -> LabResult<u64> {
    let raw = client.call(None, "getblockcount", &[])?;
    raw.parse::<u64>()
        .map_err(|e| LabError::Parse(e.to_string()))
}

pub fn get_best_block_hash<C: RpcClient>(client: &C) -> LabResult<String> {
    client.call(None, "getbestblockhash", &[])
}

pub fn inspect_network<C: RpcClient>(client: &C) -> LabResult<NetworkSnapshot> {
    let chain = get_chain(client)?;

    if chain != "regtest" {
        return Err(LabError::Rpc(format!("expected regtest, got `{chain}`")));
    }

    let block_height = get_block_height(client)?;
    let best_block_hash = get_best_block_hash(client)?;

    Ok(NetworkSnapshot {
        chain,
        block_height,
        best_block_hash,
    })
}
