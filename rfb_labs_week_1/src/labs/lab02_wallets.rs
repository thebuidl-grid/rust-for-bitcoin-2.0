use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

pub fn create_wallet<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<()> {
    client.call(None, "createwallet", &[wallet_name.to_owned()])?;
    Ok(())
}

pub fn list_wallets<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let raw = client.call(None, "listwallets", &[])?;
    let value = parse_cli_value(&raw)?;

    value
        .as_array()
        .ok_or_else(|| LabError::Parse("expected a JSON array of wallet names".to_owned()))?
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| LabError::Parse("expected a wallet name string".to_owned()))
        })
        .collect()
}

pub fn get_new_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    label: &str,
) -> LabResult<String> {
    client.call(Some(wallet_name), "getnewaddress", &[label.to_owned()])
}

pub fn address_belongs_to_wallet<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<bool> {
    let raw = client.call(Some(wallet_name), "getaddressinfo", &[address.to_owned()])?;
    let value = parse_cli_value(&raw)?;

    value
        .get("ismine")
        .and_then(Value::as_bool)
        .ok_or(LabError::MissingField("ismine"))
}
