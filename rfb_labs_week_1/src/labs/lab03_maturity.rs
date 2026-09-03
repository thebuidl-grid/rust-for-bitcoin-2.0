//! Lab 03 — demonstrate coinbase maturity.

use crate::labs::lab01_network::get_block_height;
use crate::model::{CoinbaseMaturityReport, WalletBalances};
use crate::rpc::{parse_cli_value, required_f64, RpcClient};
use crate::{LabError, LabResult};

/// Mine `count` blocks to an address and return the generated block hashes.
pub fn mine_blocks<C: RpcClient>(client: &C, address: &str, count: u64) -> LabResult<Vec<String>> {
    // TODO: call generatetoaddress with count and address.
    let call = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), address.to_string()],
    )?;
    let response = parse_cli_value(&call)?;
    response
        .as_array()
        .ok_or_else(|| LabError::Parse("expected array".to_string()))?
        .iter()
        .map(|v| {
            v.as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| LabError::Parse("expected string array".to_string()))
        })
        .collect()
}

/// Read the wallet's trusted, untrusted-pending, and immature balances.
pub fn get_balances<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<WalletBalances> {
    // TODO: call getbalances in wallet context and decode the nested `mine` object.
    let call = client.call(Some(wallet_name), "getbalances", &[])?;
    let response = parse_cli_value(&call)?;
    let mine = response
        .get("mine")
        .ok_or_else(|| LabError::MissingField("mine"))?;
    Ok(WalletBalances {
        trusted: required_f64(mine, "trusted")?,
        untrusted_pending: required_f64(mine, "untrusted_pending")?,
        immature: required_f64(mine, "immature")?,
    })
}

/// Attempt a wallet payment and return either its TXID or the Bitcoin Core error.
pub fn attempt_payment<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
    amount_btc: f64,
) -> LabResult<String> {
    use serde_json::Value;

    // Call sendtoaddress in wallet context with address and amount
    let call = client.call(
        Some(wallet_name),
        "sendtoaddress",
        &[address.to_string(), amount_btc.to_string()],
    )?;
    let response = parse_cli_value(&call)?;

    // Return the TXID string
    match response {
        Value::String(s) => Ok(s),
        _ => Err(LabError::Parse("expected txid string".to_string())),
    }
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
    let attempt_spend = match attempt_payment(client, miner_wallet, receiver_address, 1.0) {
        Ok(_) => {
            return Err(LabError::Rpc(
                "Payment should have failed but Successfully".to_string(),
            ))
        }
        Err(LabError::Rpc(msg)) => msg,
        Err(e) => return Err(e),
    };
    mine_blocks(client, miner_address, 100)?;
    let final_height = get_block_height(client)?;
    let final_balance = get_balances(client, miner_wallet)?;

    Ok(CoinbaseMaturityReport {
        height_after_first_block,
        balance_after_first_block,
        premature_spend_error: attempt_spend,
        final_height,
        final_balance,
    })
}
