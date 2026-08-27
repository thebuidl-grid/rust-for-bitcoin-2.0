//! Lab 02 — create wallets and receiving addresses.

use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

/// Create a wallet with the supplied name.
pub fn create_wallet<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<()> {
    // TODO: call createwallet with the wallet name.
    client.call(None, "createwallet", &[wallet_name.to_owned()])?;
    Ok(())
}

/// Return every wallet currently loaded by this node.
pub fn list_wallets<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    // TODO: call listwallets and decode its JSON string array.
    let raw = client.call(None, "listwallets", &[])?;
    let value = parse_cli_value(&raw)?;
    let wallets = value
        .as_array()
        .ok_or(LabError::Parse(
            "expected a JSON array of wallets".to_owned(),
        ))?
        .iter()
        .map(|entry| {
            entry.as_str().map(ToOwned::to_owned).ok_or(LabError::Parse(
                "expected wallet name to be a string".to_owned(),
            ))
        })
        .collect::<LabResult<Vec<String>>>()?;

    Ok(wallets)
}

/// Generate a labelled address in the selected wallet.
pub fn get_new_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    label: &str,
) -> LabResult<String> {
    // TODO: use wallet context and call getnewaddress with the supplied label.
    let raw = client.call(Some(wallet_name), "getnewaddress", &[label.to_owned()])?;
    let value = parse_cli_value(&raw)?;
    value
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or(LabError::Parse("expected a string address".to_owned()))
}

/// Ask the selected wallet whether it controls the supplied address.
pub fn address_belongs_to_wallet<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<bool> {
    // TODO: call getaddressinfo and return the `ismine` field.
    let raw = client.call(Some(wallet_name), "getaddressinfo", &[address.to_owned()])?;
    let value = parse_cli_value(&raw)?;
    value
        .get("ismine")
        .and_then(serde_json::Value::as_bool)
        .ok_or(LabError::MissingField("ismine"))
}
