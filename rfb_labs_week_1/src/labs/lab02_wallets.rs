//! Lab 02 — create wallets and receiving addresses.

use crate::rpc::RpcClient;
use crate::{LabResult, LabError};
use serde_json::Value;

/// Create a wallet with the supplied name.
pub fn create_wallet<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<()> {
    // TODO: call createwallet with the wallet name.
    // todo!("Lab 02: create a wallet")
     client.call(None, "createwallet", &[wallet_name.to_string()])?;
    Ok(())
}

/// Return every wallet currently loaded by this node.
pub fn list_wallets<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    // TODO: call listwallets and decode its JSON string array.
    // todo!("Lab 02: list loaded wallets")
    let raw = client.call(None, "listwallets", &[])?;
    let value: Value = serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    let wallets = value
        .as_array()
        .ok_or(LabError::MissingField("listwallets"))?
        .iter()
        .map(|w| w.as_str().map(|s| s.to_string()))
        .collect::<Option<Vec<String>>>()
        .ok_or(LabError::MissingField("listwallets"))?;

    Ok(wallets)
}

/// Generate a labelled address in the selected wallet.
pub fn get_new_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    label: &str,
) -> LabResult<String> {
    // TODO: use wallet context and call getnewaddress with the supplied label.
    // todo!("Lab 02: generate a wallet address")
    let raw = client.call(
        Some(wallet_name),
        "getnewaddress",
        &[label.to_string()],
    )?;

    Ok(raw.trim().trim_matches('"').to_string())
}

/// Ask the selected wallet whether it controls the supplied address.
pub fn address_belongs_to_wallet<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<bool> {
    // TODO: call getaddressinfo and return the `ismine` field.
    // todo!("Lab 02: verify address ownership")
     let raw = client.call(
        Some(wallet_name),
        "getaddressinfo",
        &[address.to_string()],
    )?;

    let value: Value = serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    value["ismine"]
        .as_bool()
        .ok_or(LabError::MissingField("ismine"))
}
