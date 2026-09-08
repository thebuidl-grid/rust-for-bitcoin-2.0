//! Lab 02 — create wallets and receiving addresses.

use crate::rpc::{parse_cli_value, RpcClient};
use crate::LabResult;

/// Create a wallet with the supplied name.
pub fn create_wallet<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<()> {
    // TODO: call createwallet with the wallet name.
    // todo!("Lab 02: create a wallet")
    let _ = client.call(None, "createwallet", &[wallet_name.to_string()]);

    Ok(())
}

/// Return every wallet currently loaded by this node.
pub fn list_wallets<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    // TODO: call listwallets and decode its JSON string array.
    // todo!("Lab 02: list loaded wallets")
    let result = client.call(None, "listwallets", &[]);

    match result {
        Ok(res) => {
            let parsed = parse_cli_value(&res)?;
            let vector_of_result = parsed
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_str().unwrap().to_string())
                .collect::<Vec<String>>();

            Ok(vector_of_result)
        }
        Err(err) => Err(err),
    }
}

/// Generate a labelled address in the selected wallet.
pub fn get_new_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    label: &str,
) -> LabResult<String> {
    // TODO: use wallet context and call getnewaddress with the supplied label.
    // todo!("Lab 02: generate a wallet address")
    let result = client.call(Some(wallet_name), "getnewaddress", &[label.to_string()]);

    result
}

/// Ask the selected wallet whether it controls the supplied address.
pub fn address_belongs_to_wallet<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<bool> {
    // TODO: call getaddressinfo and return the `ismine` field.
    // todo!("Lab 02: verify address ownership")
    let result = client.call(Some(wallet_name), "getaddressinfo", &[address.to_string()]);

    match result {
        Ok(res) => {
            let parsed = parse_cli_value(&res)?;
            let ismine = &parsed["ismine"];
            Ok(ismine.as_bool().unwrap())
        }
        Err(err) => Err(err),
    }
}
