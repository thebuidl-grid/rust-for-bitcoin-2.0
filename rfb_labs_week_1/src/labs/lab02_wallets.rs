//! Lab 02 — create wallets and receiving addresses.

use crate::rpc::RpcClient;
use crate::LabResult;

use crate::rpc::parse_cli_value;
use crate::LabError;
use serde_json::Value;

/// Create a wallet with the supplied name.
pub fn create_wallet<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<()> {
    client.call(None, "createwallet", &[wallet_name.to_owned()])?;
    Ok(())
}

/// Return every wallet currently loaded by this node.
pub fn list_wallets<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let raw = client.call(None, "listwallets", &[])?;
    let value = parse_cli_value(&raw)?;
    let arr = value
        .as_array()
        .ok_or_else(|| LabError::Parse("Expected array of wallets".to_owned()))?;
    let wallets = arr
        .iter()
        .map(|v| {
            v.as_str()
                .map(String::from)
                .ok_or_else(|| LabError::Parse("Expected wallet name to be string".to_owned()))
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
    let raw = client.call(Some(wallet_name), "getnewaddress", &[label.to_owned()])?;
    let value = parse_cli_value(&raw)?;
    let address = value
        .as_str()
        .ok_or_else(|| LabError::Parse("Expected address to be string".to_owned()))?;
    Ok(address.to_owned())
}

/// Ask the selected wallet whether it controls the supplied address.
pub fn address_belongs_to_wallet<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<bool> {
    let raw = client.call(Some(wallet_name), "getaddressinfo", &[address.to_owned()])?;
    let value = parse_cli_value(&raw)?;
    let ismine = value
        .get("ismine")
        .and_then(Value::as_bool)
        .ok_or_else(|| LabError::MissingField("ismine"))?;
    Ok(ismine)
}
