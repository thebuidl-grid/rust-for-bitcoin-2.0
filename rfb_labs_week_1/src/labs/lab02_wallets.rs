//! Lab 02 — create wallets and receiving addresses.

use crate::rpc::parse_cli_value;
use crate::rpc::RpcClient;
use crate::{LabError, LabResult};

/// Create a wallet with the supplied name.
pub fn create_wallet<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<()> {
    client.call(None, "createwallet", &[wallet_name.to_owned()])?;
    Ok(())
}

/// Return every wallet currently loaded by this node.
pub fn list_wallets<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let response = client.call(None, "listwallets", &[])?;
    let value = parse_cli_value(&response)?;

    value
        .as_array()
        .ok_or(LabError::Parse("wallet list must be an array".to_string()))?
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or(LabError::Parse("wallet name must be a string".to_string()))
        })
        .collect()
}

/// Generate a labelled address in the selected wallet.
pub fn get_new_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    label: &str,
) -> LabResult<String> {
    let response = client.call(Some(wallet_name), "getnewaddress", &[label.to_owned()])?;
    parse_cli_value(&response)?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or(LabError::Parse("new address must be a string".to_string()))
}

/// Ask the selected wallet whether it controls the supplied address.
pub fn address_belongs_to_wallet<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<bool> {
    let response = client.call(Some(wallet_name), "getaddressinfo", &[address.to_owned()])?;
    let value = parse_cli_value(&response)?;

    value
        .get("ismine")
        .and_then(serde_json::Value::as_bool)
        .ok_or(LabError::MissingField("ismine"))
}
