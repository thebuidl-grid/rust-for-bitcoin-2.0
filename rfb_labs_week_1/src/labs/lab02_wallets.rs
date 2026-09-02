//! Lab 02 — create wallets and receiving addresses.

use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

/// Create a wallet with the supplied name.
pub fn create_wallet<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<()> {
    client.call(None, "createwallet", &[wallet_name.to_owned()])?;
    Ok(())
}

/// Return every wallet currently loaded by this node.
pub fn list_wallets<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let raw = client.call(None, "listwallets", &[])?;
    let wallets: Vec<String> = serde_json::from_str(raw.trim())?;
    Ok(wallets)
}

/// Generate a labelled address in the selected wallet.
pub fn get_new_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    label: &str,
) -> LabResult<String> {
    // Address generation is wallet-scoped, so this call needs `-rpcwallet=<name>`.
    let raw = client.call(Some(wallet_name), "getnewaddress", &[label.to_owned()])?;
    Ok(raw.trim().to_owned())
}

/// Ask the selected wallet whether it controls the supplied address.
pub fn address_belongs_to_wallet<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<bool> {
    let raw = client.call(Some(wallet_name), "getaddressinfo", &[address.to_owned()])?;
    parse_cli_value(&raw)?
        .get("ismine")
        .and_then(Value::as_bool)
        .ok_or(LabError::MissingField("ismine"))
}
