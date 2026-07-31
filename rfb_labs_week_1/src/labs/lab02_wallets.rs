//! Lab 02 — create wallets and receiving addresses.

use crate::rpc::RpcClient;
use crate::LabResult;
use crate::error::LabError;
use serde_json::Value;

/// Create a wallet with the supplied name.
pub fn create_wallet<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<()> {
    // TODO: call createwallet with the wallet name.
client.call(None, "createwallet", &[wallet_name.to_string()])?;
    Ok(())
}

/// Return every wallet currently loaded by this node.
pub fn list_wallets<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    // TODO: call listwallets and decode its JSON string array.
    let res = client.call(None, "listwallets", &[])?;

    let wallets: Vec<String> = serde_json::from_str(&res)
        .map_err(|e| LabError::Rpc(format!("Failed to parse listwallets response: {}", e)))?;

    Ok(wallets)
    // todo!("Lab 02: list loaded wallets")
}

/// Generate a labelled address in the selected wallet.
pub fn get_new_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    label: &str,
) -> LabResult<String> {
    // TODO: use wallet context and call getnewaddress with the supplied label.
    let res = client.call(Some(wallet_name), "getnewaddress", &[label.to_string()])?;

    // `getnewaddress` returns a string (the address), trim quotes or whitespace if stringified
    Ok(res.trim().trim_matches('"').to_string())
    // todo!("Lab 02: generate a wallet address")
}

/// Ask the selected wallet whether it controls the supplied address.
pub fn address_belongs_to_wallet<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<bool> {
    // TODO: call getaddressinfo and return the `ismine` field.
    let res = client.call(Some(wallet_name), "getaddressinfo", &[address.to_string()])?;

    let json: Value = serde_json::from_str(&res)
        .map_err(|e| LabError::Rpc(format!("Failed to parse getaddressinfo response: {}", e)))?;

    json["ismine"]
        .as_bool()
        .ok_or_else(|| LabError::Rpc("Missing 'ismine' field in getaddressinfo output".into()))

    // todo!("Lab 02: verify address ownership")
}
