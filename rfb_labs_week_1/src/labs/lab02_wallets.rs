//! Lab 02 — create wallets and receiving addresses.

use std::fmt::format;

use serde_json::Value;

use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

/// Create a wallet with the supplied name.
pub fn create_wallet<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<()> {
    let raw = client.call(None, "createwallet", &[wallet_name.to_string()])?;
    let _ = parse_cli_value(&raw)?;
    Ok(())
}

/// Return every wallet currently loaded by this node.
pub fn list_wallets<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let raw = client.call(None, "listwallets", &[])?;
    let value = parse_cli_value(&raw)?;

    match value {
        Value::Array(arr) => arr
            .iter()
            .map(|v| {
                v.as_str()
                    .map(ToOwned::to_owned)
                    .ok_or(LabError::Parse("expected String listwallets".to_string()))
            })
            .collect(),
        other => Err(LabError::Parse(format!("expected array, got {other}"))),
    }
}

/// Generate a labelled address in the selected wallet.
pub fn get_new_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    label: &str,
) -> LabResult<String> {
    // TODO: use wallet context and call getnewaddress with the supplied label.
    let raw = client.call(Some(wallet_name), "getnewaddress", &[label.to_string()])?;
    let value = parse_cli_value(&raw)?;

    match value {
        serde_json::Value::String(addr) => Ok(addr),
        other => Err(crate::LabError::Parse(format!(
            "expected address, got {other}"
        ))),
    }
}

/// Ask the selected wallet whether it controls the supplied address.
pub fn address_belongs_to_wallet<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<bool> {
    let raw = client.call(Some(wallet_name), "getaddressinfo", &[address.to_string()])?;
    let value = parse_cli_value(&raw)?;

    value
        .get("ismine")
        .and_then(Value::as_bool)
        .ok_or(LabError::MissingField("ismine"))
}
