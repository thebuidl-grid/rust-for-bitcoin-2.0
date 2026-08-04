//! Lab 02 — create wallets and receiving addresses.

use serde_json::Value;

use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::{LabError, LabResult};

/// Create a wallet with the supplied name.
pub fn create_wallet<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<()> {
    client.call(None, "createwallet", &[wallet_name.to_string()])?;
    //client.call(None, "loadwallet", &[wallet_name.to_string()])?;
    Ok(())
}

/// Return every wallet currently loaded by this node.
pub fn list_wallets<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let raw = client.call(None, "listwallets", &[])?;
    let value = parse_cli_value(&raw)?;
    let wallets = value
        .as_array()
        .ok_or(LabError::Parse("expected JSON array".to_string()))?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_string)
                .ok_or(LabError::Parse("expected string".to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(wallets)
}
/// Generate a labelled address in the selected wallet.
pub fn get_new_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    label: &str,
) -> LabResult<String> {
    client.call(Some(wallet_name), "getnewaddress", &[label.to_string()])
}

/// Ask the selected wallet whether it controls the supplied address.
pub fn address_belongs_to_wallet<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<bool> {
    let raw = client.call(Some(wallet_name), "getaddressinfo", &[address.to_string()])?;
    let value = parse_cli_value(&raw)?;

    match value.get("ismine").and_then(Value::as_bool) {
        Some(b) => Ok(b),
        None => Err(LabError::MissingField("ismine")),
    }
}
