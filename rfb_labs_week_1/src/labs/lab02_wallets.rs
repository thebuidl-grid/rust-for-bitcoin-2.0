//! Lab 02 — create wallets and receiving addresses.

use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::{LabError, LabResult};

/// Create a wallet with the supplied name.
pub fn create_wallet<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<()> {
    client.call(None, "createwallet", &[wallet_name.to_string()])?;
    Ok(())
}

/// Return every wallet currently loaded by this node.
pub fn list_wallets<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let response = client.call(None, "listwallets", &[])?;
    let value = parse_cli_value(&response)?;

    let wallets = value
        .as_array()
        .ok_or(LabError::Parse("expected wallet list".to_string()))?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    Ok(wallets)
}

/// Generate a labelled address in the selected wallet.
pub fn get_new_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    label: &str,
) -> LabResult<String> {
    client.call(
        Some(wallet_name),
        "getnewaddress",
        &[label.to_string()],
    )
}

/// Ask the selected wallet whether it controls the supplied address.
pub fn address_belongs_to_wallet<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<bool> {
    let response = client.call(
        Some(wallet_name),
        "getaddressinfo",
        &[address.to_string()],
    )?;

    let value = parse_cli_value(&response)?;

    value
        .get("ismine")
        .and_then(|v| v.as_bool())
        .ok_or(LabError::Parse("expected ismine".to_string()))
}