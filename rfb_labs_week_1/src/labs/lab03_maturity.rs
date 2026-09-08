//! Lab 03 — demonstrate coinbase maturity.

use crate::labs::lab01_network::get_block_height;
use crate::model::{CoinbaseMaturityReport, WalletBalances};
use crate::rpc::{RpcClient, parse_cli_value, required_f64};
use crate::{LabError, LabResult};

/// Mine `count` blocks to an address and return the generated block hashes.
pub fn mine_blocks<C: RpcClient>(client: &C, address: &str, count: u64) -> LabResult<Vec<String>> {
    // TODO: call generatetoaddress with count and address.
    let raw_info = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), address.to_string()]
    )?;
    let value = parse_cli_value(&raw_info)?;
    let array = value.as_array().ok_or(LabError::MissingField("generatetoaddress"))?;
    array.iter().map(|entry|{
        entry.as_str()
        .map(ToOwned::to_owned)
        .ok_or(LabError::MissingField("generatetoaddress[]"))
        
    })
    .collect()
}

/// Read the wallet's trusted, untrusted-pending, and immature balances.
pub fn get_balances<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<WalletBalances> {
    // TODO: call getbalances in wallet context and decode the nested `mine` object.
    let raw_info = client.call(Some(wallet_name), "getbalances", &[])?;
    let value = parse_cli_value(&raw_info)?;
    let mine = value.get("mine").ok_or(LabError::MissingField("mine"))?;

    Ok(WalletBalances { trusted: required_f64(mine, "trusted")?,
     untrusted_pending: required_f64(mine, "untrusted_pending")?,
      immature: required_f64(mine, "immature")?, })
}

/// Attempt a wallet payment and return either its TXID or the Bitcoin Core error.
pub fn attempt_payment<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
    amount_btc: f64,
) -> LabResult<String> {
    // TODO: call sendtoaddress. Do not hide an insufficient-funds RPC error.
    let raw_info = client.call(
        Some(wallet_name),
        "sendtoaddress",
        &[address.to_string(), amount_btc.to_string()],
    )?;
    let value = parse_cli_value(&raw_info)?;
    value
    .as_str()
    .map(ToOwned::to_owned)
    .ok_or(LabError::MissingField("txid"))
}

/// Mine one block, prove the reward is immature, then mine 100 more blocks.
pub fn demonstrate_coinbase_maturity<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    miner_address: &str,
    receiver_address: &str,
) -> LabResult<CoinbaseMaturityReport> {
    // TODO:
    // 1. Mine one block.
    // 2. Record height and balances.
    // 3. Attempt a 1 BTC payment and capture its error text.
    // 4. Mine 100 more blocks.
    // 5. Record final height and balances.
    mine_blocks(client, miner_address, 1)?;  

    let height_after_first_block = get_block_height(client)?;
    let balance_after_first_block = get_balances(client, miner_wallet)?;


    let premature_spend_error = match attempt_payment(client, miner_wallet, receiver_address, 1.0) {
    Ok(txid) => {
        return Err(LabError::Rpc(format!(
            "expected premature coinbase spend to fail, but it succeeded with txid {txid}"
        )))
    }
    Err(LabError::Rpc(message)) => message,
    Err(other) => return Err(other),
};

    mine_blocks(client, miner_address, 100)?;  // <- must be 100 here, not 1

    let final_height = get_block_height(client)?;
    let final_balance = get_balances(client, miner_wallet)?;

    Ok(CoinbaseMaturityReport {
        height_after_first_block,
        balance_after_first_block,
        premature_spend_error,
        final_height,
        final_balance,
    })
}
