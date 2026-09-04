//! Lab 01 — build and verify a regtest network.

use serde_json::Value;

use crate::model::NetworkSnapshot;
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

/// Ask Bitcoin Core for the active chain name.
pub fn get_chain<C: RpcClient>(client: &C) -> LabResult<String> {
    // Send the RPC request and keep the raw output as a string.
    let raw = client.call(None, "getblockchaininfo", &[])?;
    // Parse the raw string into JSON so we can inspect the response.
    let result = parse_cli_value(&raw)?;

    // Read the "chain" field and return it as a Rust String.
    result
        .get("chain")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or(LabError::MissingField("chain"))
}

/// Ask Bitcoin Core for the current block height and convert it to u64.
pub fn get_block_height<C: RpcClient>(client: &C) -> LabResult<u64> {
    // Send the RPC request for the current tip height.
    let raw = client.call(None, "getblockcount", &[])?;
    // Parse the response into JSON.
    let result = parse_cli_value(&raw)?;

    // The response may come back as a JSON number or as a string containing a number,
    // so we handle both forms.
    match result {
        Value::Number(number) => number
            .as_u64()
            .ok_or_else(|| LabError::Parse("expected an unsigned block height".to_owned())),
        Value::String(text) => text
            .parse::<u64>()
            .map_err(|error| LabError::Parse(error.to_string())),
        other => Err(LabError::Parse(format!(
            "expected a numeric height, got {other}"
        ))),
    }
}

/// Ask Bitcoin Core for the current best-block hash.
pub fn get_best_block_hash<C: RpcClient>(client: &C) -> LabResult<String> {
    // Send the RPC request to get the current best-block hash.
    let raw = client.call(None, "getbestblockhash", &[])?;
    // Parse the response into JSON.
    let result = parse_cli_value(&raw)?;

    // The hash is returned as a string, so we return it directly.
    match result {
        Value::String(hash) => Ok(hash),
        other => Err(LabError::Parse(format!(
            "expected a best-block hash string, got {other}"
        ))),
    }
}

/// Gather the chain, height, and best-block hash into one verified snapshot.
pub fn inspect_network<C: RpcClient>(client: &C) -> LabResult<NetworkSnapshot> {
    // First confirm the node is on regtest.
    let chain = get_chain(client)?;
    if chain != "regtest" {
        return Err(LabError::Rpc("network is not regtest".to_owned()));
    }

    // Once the network is verified, collect the rest of the data.
    let block_height = get_block_height(client)?;
    let best_block_hash = get_best_block_hash(client)?;

    Ok(NetworkSnapshot {
        chain,
        block_height,
        best_block_hash,
    })
}
