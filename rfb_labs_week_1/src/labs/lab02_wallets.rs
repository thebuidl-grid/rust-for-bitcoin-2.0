//! Lab 02 — create wallets and receiving addresses.

use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

/// Create a wallet with the supplied name.
pub fn create_wallet<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<()> {
    client.call(None, "createwallet", &[wallet_name.to_string()])?;
    Ok(())
}

/// Return every wallet currently loaded by this node.
pub fn list_wallets<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let raw = client.call(None, "listwallets", &[])?;
    let val = parse_cli_value(&raw)?;
    serde_json::from_value(val).map_err(|e| LabError::Parse(e.to_string()))
}

/// Generate a labelled address in the selected wallet.
pub fn get_new_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    label: &str,
) -> LabResult<String> {
    let raw = client.call(Some(wallet_name), "getnewaddress", &[label.to_string()])?;
    let val = parse_cli_value(&raw)?;
    match val {
        serde_json::Value::String(s) => Ok(s),
        _ => Err(LabError::Parse("invalid address".to_string())),
    }
}

/// Ask the selected wallet whether it controls the supplied address.
pub fn address_belongs_to_wallet<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<bool> {
    let raw = client.call(Some(wallet_name), "getaddressinfo", &[address.to_string()])?;
    let val = parse_cli_value(&raw)?;
    val.get("ismine")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| LabError::MissingField("ismine"))
}
