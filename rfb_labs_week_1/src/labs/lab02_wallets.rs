//! Lab 02 — create wallets and receiving addresses.

use crate::rpc::{parse_cli_value, required_string, RpcClient};
use serde_json::Value;
use crate::{LabError, LabResult};

/// Create a wallet with the supplied name.
pub fn create_wallet<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<()> {
    client.call(None, "createwallet", &[wallet_name.to_string()])?;
    Ok(())
}

/// Return every wallet currently loaded by this node.
pub fn list_wallets<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let call = client.call(None, "listwallets", &[])?;
    let cli_response = parse_cli_value(&call)?;

    cli_response.as_array()
    .ok_or_else(|| LabError::Parse("expected array".to_string()))?
    .iter()
    .map(|v| {
        v.as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| LabError::Parse("expected string array".to_string()))
    })
    .collect()

}

/// Generate a labelled address in the selected wallet.
pub fn get_new_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    label: &str,
) -> LabResult<String> {
    let call = client.call(Some(wallet_name), "getnewaddress", &[label.to_string()])?;
    let cli_response = parse_cli_value(&call)?;

    match cli_response {
        Value::String(s) => Ok(s),
        _ => Err(LabError::Parse("expected string".to_string())),
    }
}

/// Ask the selected wallet whether it controls the supplied address.
pub fn address_belongs_to_wallet<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<bool> {
    let call = client.call(Some(wallet_name), "getaddressinfo", &[address.to_string()])?;
    let response = parse_cli_value(&call)?;

    response
        .get("ismine")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| LabError::MissingField("ismine"))
}
