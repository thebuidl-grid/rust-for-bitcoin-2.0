use crate::error::LabError;
use crate::model::NetworkSnapshot;
use crate::rpc::RpcClient;
use serde_json::Value;

/// Fetches the chain name (e.g. "regtest") from getblockchaininfo RPC.
pub fn get_chain(client: &dyn RpcClient) -> Result<String, LabError> {
    // 3 arguments required: wallet (None), method ("getblockchaininfo"), params (&[])
    let res = client.call(None, "getblockchaininfo", &[])?;

    // Parse the returned JSON string into serde_json::Value
    let json: Value = serde_json::from_str(&res)
        .map_err(|e| LabError::Rpc(format!("Failed to parse getblockchaininfo JSON: {}", e)))?;

    json["chain"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| LabError::Rpc("Missing 'chain' field in getblockchaininfo".into()))
}

/// Fetches the current block height from getblockcount RPC.
pub fn get_block_height(client: &dyn RpcClient) -> Result<u64, LabError> {
    // getblockcount returns a raw string representation of the height (e.g., "101")
    let res = client.call(None, "getblockcount", &[])?;

    res.trim()
        .parse::<u64>()
        .map_err(|e| LabError::Rpc(format!("Invalid block height format: {}", e)))
}

/// Fetches the best block hash from getbestblockhash RPC.
pub fn get_best_block_hash(client: &dyn RpcClient) -> Result<String, LabError> {
    let res = client.call(None, "getbestblockhash", &[])?;

    Ok(res.trim().trim_matches('"').to_string())
}

/// Aggregates network status into a NetworkSnapshot struct.
pub fn inspect_network(client: &dyn RpcClient) -> Result<NetworkSnapshot, LabError> {
    let chain = get_chain(client)?;
    let block_height = get_block_height(client)?;
    let best_block_hash = get_best_block_hash(client)?;

    Ok(NetworkSnapshot {
        chain,
        block_height,
        best_block_hash,
    })
}