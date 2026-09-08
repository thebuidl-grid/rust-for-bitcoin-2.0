//! Lab 02 — create wallets and receiving addresses.

use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

/// Create a wallet with the supplied name.
pub fn create_wallet<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<()> {
    client.call(None, "createwallet", &[wallet_name.to_owned()])?;
    Ok(())
}

/// Return every wallet currently loaded by this node.
pub fn list_wallets<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let response = client.call(None, "listwallets", &[])?;
    let wallets = serde_json::from_str(&response)?;
    Ok(wallets)
}

/// Generate a labelled address in the selected wallet.
pub fn get_new_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    label: &str,
) -> LabResult<String> {
    let response = client.call(Some(wallet_name), "getnewaddress", &[label.to_owned()])?;

    Ok(response)
}

/// Ask the selected wallet whether it controls the supplied address.
pub fn address_belongs_to_wallet<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<bool> {
    let response = client.call(Some(wallet_name), "getaddressinfo", &[address.to_owned()])?;

    let parsed_response = parse_cli_value(&response)?;

    parsed_response
        .get("ismine")
        .and_then(|value| value.as_bool())
        .ok_or(LabError::MissingField("ismine"))
}
