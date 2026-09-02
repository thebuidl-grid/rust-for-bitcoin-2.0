//! Lab 02 — create wallets and receiving addresses.

use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::{LabError, LabResult};

/// Create a wallet with the supplied name.
pub fn create_wallet<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<()> {
    // 1. Call `createwallet` passing the wallet_name as a string parameter
    let raw = client.call(None, "createwallet", &[wallet_name.into()])?;
    
    // 2. Parse the output to ensure the command succeeded
    let _value = parse_cli_value(&raw)?;
    
    Ok(())
}

/// Return every wallet currently loaded by this node.
pub fn list_wallets<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    // 1. Call `listwallets` with no arguments
    let raw = client.call(None, "listwallets", &[])?;
    let value = parse_cli_value(&raw)?;

    // 2. Convert the JSON array into Vec<String>
    let wallets = value
        .as_array()
        .ok_or_else(|| LabError::Parse("listwallets did not return an array".to_owned()))?
        .iter()
        .filter_map(|val| val.as_str().map(ToOwned::to_owned))
        .collect();

    Ok(wallets)
}

/// Generate a labelled address in the selected wallet.
pub fn get_new_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    label: &str,
) -> LabResult<String> {
    // 1. Specify the wallet name as the wallet target context (Some(wallet_name))
    // 2. Pass the label string as an argument to `getnewaddress`
    let raw = client.call(Some(wallet_name), "getnewaddress", &[label.into()])?;
    let value = parse_cli_value(&raw)?;

    // 3. Extract the address string
    value
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| LabError::Parse("getnewaddress did not return a string".to_owned()))
}

/// Ask the selected wallet whether it controls the supplied address.
pub fn address_belongs_to_wallet<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<bool> {
    // 1. Call `getaddressinfo` inside the context of `wallet_name`
    let raw = client.call(Some(wallet_name), "getaddressinfo", &[address.into()])?;
    let value = parse_cli_value(&raw)?;

    // 2. Extract the `ismine` boolean field
    value["ismine"]
        .as_bool()
        .ok_or_else(|| LabError::Parse("getaddressinfo `ismine` was missing or not a boolean".to_owned()))
}