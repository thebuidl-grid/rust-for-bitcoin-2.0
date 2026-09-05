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
    let call = client.call(None, "listwallets", &[])?;
    let val = parse_cli_value(&call)?;

    serde_json::from_value::<Vec<String>>(val).map_err(Into::into)
}

/// Generate a labelled address in the selected wallet.
pub fn get_new_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    label: &str,
) -> LabResult<String> {
    // TODO: use wallet context and call getnewaddress with the supplied label.
    let call = client.call(Some(wallet_name), "getnewaddress", &[label.to_string()])?;
    let val = parse_cli_value(&call)?;

    val.as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| LabError::Parse("getnewaddress response is not a string".to_string()))
}

/// Ask the selected wallet whether it controls the supplied address.
pub fn address_belongs_to_wallet<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<bool> {
    // TODO: call getaddressinfo and return the `ismine` field.
    let call = client.call(Some(wallet_name), "getaddressinfo", &[address.to_string()])?;
    let val = parse_cli_value(&call)?;

    val.get("ismine")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| LabError::Parse("getaddressinfo response missing ismine".to_string()))
}
