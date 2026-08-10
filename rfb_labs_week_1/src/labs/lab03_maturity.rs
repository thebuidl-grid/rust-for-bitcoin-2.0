//! Lab 03 — demonstrate coinbase maturity.

use crate::model::{CoinbaseMaturityReport, WalletBalances};
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

/// Mine `count` blocks to an address and return block hashes.
pub fn mine_blocks<C: RpcClient>(client: &C, address: &str, count: u64) -> LabResult<Vec<String>> {
    let params = vec![count.to_string(), address.to_string()];

    let raw = client.call(None, "generatetoaddress", &params)?;

    let value = parse_cli_value(&raw)?;

    let blocks = value
        .as_array()
        .ok_or(LabError::Parse("Expected block hash array".to_string()))?
        .iter()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect();

    Ok(blocks)
}

/// Read wallet balances.
pub fn get_balances<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<WalletBalances> {
    let raw = client.call(Some(wallet_name), "getbalances", &[])?;

    let value = parse_cli_value(&raw)?;

    let mine = value.get("mine").ok_or(LabError::MissingField("mine"))?;

    Ok(WalletBalances {
        trusted: mine.get("trusted").and_then(Value::as_f64).unwrap_or(0.0),

        untrusted_pending: mine
            .get("untrusted_pending")
            .and_then(Value::as_f64)
            .unwrap_or(0.0),

        immature: mine.get("immature").and_then(Value::as_f64).unwrap_or(0.0),
    })
}

/// Attempt wallet payment.
pub fn attempt_payment<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
    amount_btc: f64,
) -> LabResult<String> {
    let params = vec![address.to_string(), amount_btc.to_string()];

    client.call(Some(wallet_name), "sendtoaddress", &params)
}

/// Demonstrate coinbase maturity.
pub fn demonstrate_coinbase_maturity<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    miner_address: &str,
    receiver_address: &str,
) -> LabResult<CoinbaseMaturityReport> {
    // Mine first block
    mine_blocks(client, miner_address, 1)?;

    // Height after first block
    let height_after_first_block = client
        .call(None, "getblockcount", &[])?
        .parse::<u64>()
        .map_err(|e| LabError::Parse(e.to_string()))?;

    // Balance before maturity
    let balance_after_first_block = get_balances(client, miner_wallet)?;

    // Try spending immature coinbase
    let premature_spend_error = match attempt_payment(client, miner_wallet, receiver_address, 1.0) {
        Ok(_) => String::new(),

        Err(LabError::Rpc(msg)) => msg,

        Err(e) => e.to_string(),
    };

    // Mine 100 more blocks
    mine_blocks(client, miner_address, 100)?;

    let final_height = client
        .call(None, "getblockcount", &[])?
        .parse::<u64>()
        .map_err(|e| LabError::Parse(e.to_string()))?;

    let final_balance = get_balances(client, miner_wallet)?;

    Ok(CoinbaseMaturityReport {
        height_after_first_block,
        balance_after_first_block,
        premature_spend_error,
        final_height,
        final_balance,
    })
}
