//! Lab 03 — demonstrate coinbase maturity.

use crate::rpc::parse_cli_value;
use crate::model::{CoinbaseMaturityReport, WalletBalances};
use crate::rpc::RpcClient;
use crate::{LabError, LabResult};

/// Mine `count` blocks to an address and return the generated block hashes.
pub fn mine_blocks<C: RpcClient>(client: &C, address: &str, count: u64) -> LabResult<Vec<String>> {
    let response = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), address.to_owned()],
    )?;
    let value = parse_cli_value(&response)?;

    value
        .as_array()
        .ok_or(LabError::Parse("generated blocks must be an array".to_string()))?
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or(LabError::Parse("block hash must be a string".to_string()))
        })
        .collect()
}

/// Read the wallet's trusted, untrusted-pending, and immature balances.
pub fn get_balances<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<WalletBalances> {
    let response = client.call(Some(wallet_name), "getbalances", &[])?;
    let value = parse_cli_value(&response)?;
    let mine = value
        .get("mine")
        .ok_or(LabError::MissingField("mine"))?;

    Ok(WalletBalances {
        trusted: mine
            .get("trusted")
            .and_then(serde_json::Value::as_f64)
            .ok_or(LabError::MissingField("trusted"))?,
        untrusted_pending: mine
            .get("untrusted_pending")
            .and_then(serde_json::Value::as_f64)
            .ok_or(LabError::MissingField("untrusted_pending"))?,
        immature: mine
            .get("immature")
            .and_then(serde_json::Value::as_f64)
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
    let response = client.call(
        Some(wallet_name),
        "sendtoaddress",
        &[address.to_owned(), amount_btc.to_string()],
    )?;

    parse_cli_value(&response)?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or(LabError::Parse("transaction id must be a string".to_string()))
}

/// Mine one block, prove the reward is immature, then mine 100 more blocks.
pub fn demonstrate_coinbase_maturity<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    miner_address: &str,
    receiver_address: &str,
) -> LabResult<CoinbaseMaturityReport> {
    mine_blocks(client, miner_address, 1)?;
    let height_after_first_block = client
        .call(None, "getblockcount", &[])
        .and_then(|response| parse_cli_value(&response))?
        .as_u64()
        .ok_or(LabError::Parse("block height must be a number".to_string()))?;
    let balance_after_first_block = get_balances(client, miner_wallet)?;
    let premature_spend_error = attempt_payment(client, miner_wallet, receiver_address, 1.0)
        .err()
        .ok_or(LabError::Parse(
            "expected an insufficient-funds RPC error".to_string(),
        ))?;

    mine_blocks(client, miner_address, 100)?;
    let final_height = client
        .call(None, "getblockcount", &[])
        .and_then(|response| parse_cli_value(&response))?
        .as_u64()
        .ok_or(LabError::Parse("block height must be a number".to_string()))?;
    let final_balance = get_balances(client, miner_wallet)?;

    Ok(CoinbaseMaturityReport {
        height_after_first_block,
        balance_after_first_block,
        premature_spend_error: format!("{premature_spend_error}"),
        final_height,
        final_balance,
    })
}
