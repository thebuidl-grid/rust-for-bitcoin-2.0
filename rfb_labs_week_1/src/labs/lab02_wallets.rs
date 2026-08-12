//! Lab 02 — create wallets and receiving addresses.

use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

/// Create a wallet with the supplied name.
pub fn create_wallet<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<()> {
    client.call(None, "createwallet", &[wallet_name.to_string()])?;
    Ok(()) // Ignore the returned JSON, we just want to create the wallet
}

/// Return every wallet currently loaded by this node.
pub fn list_wallets<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let raw = client.call(None, "listwallets", &[])?;
    let val = parse_cli_value(&raw)?;

    val.as_array()
        .ok_or_else(|| LabError::Parse("Expected array for listwallets".into()))?
        .iter()
        .map(|item| {
            item.as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| LabError::Parse("Invalid wallet name string".into()))
        })
        .collect()
}

/// Generate a labelled address in the selected wallet.
pub fn get_new_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    label: &str,
) -> LabResult<String> {
    let raw = client.call(Some(wallet_name), "getnewaddress", &[label.to_string()])?;
    let val = parse_cli_value(&raw)?;

    val.as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| LabError::Parse("Invalid address returned".into()))
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
        .and_then(|v| v.as_bool())
        .ok_or_else(|| LabError::Parse("Missing or invalid 'ismine' field".into()))
}
