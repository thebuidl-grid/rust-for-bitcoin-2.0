//! Lab 02 — create wallets and receiving addresses.

use crate::rpc::{parse_cli_value, RpcClient};
use crate::LabResult;
use serde_json::Value;

/// Create a wallet with the supplied name.
pub fn create_wallet<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<()> {
    let params = vec![wallet_name.to_string()];

    client.call(None, "createwallet", &params)?;

    Ok(())
}

/// Return every wallet currently loaded by this node.
pub fn list_wallets<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let raw = client.call(None, "listwallets", &[])?;

    let value = parse_cli_value(&raw)?;

    let wallets = value
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect();

    Ok(wallets)
}

/// Generate a labelled address in the selected wallet.
pub fn get_new_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    label: &str,
) -> LabResult<String> {
    let params = vec![label.to_string()];

    let raw = client.call(Some(wallet_name), "getnewaddress", &params)?;

    Ok(raw)
}

/// Ask the selected wallet whether it controls the supplied address.
pub fn address_belongs_to_wallet<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<bool> {
    let params = vec![address.to_string()];

    let raw = client.call(Some(wallet_name), "getaddressinfo", &params)?;

    let value = parse_cli_value(&raw)?;

    Ok(value
        .get("ismine")
        .and_then(Value::as_bool)
        .unwrap_or(false))
}
