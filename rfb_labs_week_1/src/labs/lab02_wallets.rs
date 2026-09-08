//! Lab 02 — create wallets and receiving addresses.

use crate::rpc::{RpcClient, parse_cli_value};
use crate::{LabError, LabResult};

/// Create a wallet with the supplied name.
pub fn create_wallet<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<()> {
    // TODO: call createwallet with the wallet name.
    client.call(None, "createwallet", &[wallet_name.to_string()])?;
    Ok(())
}

/// Return every wallet currently loaded by this node.
pub fn list_wallets<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    // TODO: call listwallets and decode its JSON string array.
    let raw_info = client.call(None, "listwallets", &[])?;
    let value = parse_cli_value(&raw_info)?;
    let array = value
    .as_array()
    .ok_or(LabError::MissingField("listwallets"))?;

    array.iter().map(|entry| {
        entry.as_str()
        .map(ToOwned::to_owned)
        .ok_or(LabError::MissingField("listwallets[]"))
    })
    .collect()
}

/// Generate a labelled address in the selected wallet.
pub fn get_new_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    label: &str,
) -> LabResult<String> {
    // TODO: use wallet context and call getnewaddress with the supplied label.
    let raw_info = client.call(
        Some(wallet_name),
        "getnewaddress",
        &[label.to_string()]
    )?;
    let value = parse_cli_value(&raw_info)?;
    value.as_str().map(ToOwned::to_owned).ok_or(LabError::MissingField("address"))
}

/// Ask the selected wallet whether it controls the supplied address.
pub fn address_belongs_to_wallet<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<bool> {
    // TODO: call getaddressinfo and return the `ismine` field.
    let raw_info = client.call(
        Some(wallet_name),
        "getaddressinfo",
        &[address.to_string()]
    )?;
    let value = parse_cli_value(&raw_info)?;
    value.get("ismine")
    .and_then(serde_json::Value::as_bool)
    .ok_or(LabError::MissingField("ismine"))
}
