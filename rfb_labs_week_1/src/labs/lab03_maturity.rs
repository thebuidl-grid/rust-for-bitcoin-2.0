//! Lab 03 — demonstrate coinbase maturity.

use serde_json::Value;

use crate::labs::lab01_network::get_block_height;
use crate::model::{CoinbaseMaturityReport, WalletBalances};
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

/// Mine `count` blocks to an address and return the generated block hashes.
pub fn mine_blocks<C: RpcClient>(client: &C, address: &str, count: u64) -> LabResult<Vec<String>> {
    let raw = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), address.to_string()],
    )?;
    let value = parse_cli_value(&raw)?;

    match value {
        Value::Array(items) => items
            .iter()
            .map(|item| {
                item.as_str()
                    .map(ToOwned::to_owned)
                    .ok_or(LabError::Parse("expected string block hash".to_string()))
            })
            .collect(),
        other => Err(LabError::Parse(format!("expected array, got {other}"))),
    }
}

/// Read the wallet's trusted, untrusted-pending, and immature balances.
pub fn get_balances<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<WalletBalances> {
    let raw = client.call(Some(wallet_name), "getbalances", &[])?;
    let value = parse_cli_value(&raw)?;

    let mine = value
        .get("mine")
        .and_then(Value::as_object)
        .ok_or(LabError::MissingField("mine"))?;

    Ok(WalletBalances {
        trusted: mine
            .get("trusted")
            .and_then(Value::as_f64)
            .ok_or(LabError::MissingField("trusted"))?,
        untrusted_pending: mine
            .get("untrusted_pending")
            .and_then(Value::as_f64)
            .ok_or(LabError::MissingField("untrusted_pending"))?,
        immature: mine
            .get("immature")
            .and_then(Value::as_f64)
            .ok_or(LabError::MissingField("immature"))?,
    })
}

/// Attempt a wallet payment and return either its TXID or the Bitcoin Core error.
pub fn attempt_payment<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
    amount_btc: f64,
) -> LabResult<String> {
    let raw = client.call(
        Some(wallet_name),
        "sendtoaddress",
        &[address.to_string(), amount_btc.to_string()],
    )?;
    let value = parse_cli_value(&raw)?;

    match value {
        Value::String(txid) => Ok(txid),
        other => Err(LabError::Parse(format!(
            "expected txid string, got {other}"
        ))),
    }
}

/// Mine one block, prove the reward is immature, then mine 100 more blocks.
pub fn demonstrate_coinbase_maturity<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    miner_address: &str,
    receiver_address: &str,
) -> LabResult<CoinbaseMaturityReport> {
    mine_blocks(client, miner_address, 1)?;
    let height_after_first_block = get_block_height(client)?;
    let balance_after_first_block = get_balances(client, miner_wallet)?;

    let premature_spend_error = match attempt_payment(client, miner_wallet, receiver_address, 1.0) {
        Ok(_) => return Err(LabError::Rpc("expected payment to fail".to_owned())),
        Err(LabError::Rpc(message)) => message,
        Err(other) => return Err(other),
    };

    mine_blocks(client, miner_address, 100)?;
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
