//! Lab 03 — demonstrate coinbase maturity.

use crate::model::{CoinbaseMaturityReport, WalletBalances};
use crate::rpc::RpcClient;
use crate::LabResult;

use crate::rpc::parse_cli_value;
use crate::LabError;
use serde_json::Value;

/// Mine `count` blocks to an address and return the generated block hashes.
pub fn mine_blocks<C: RpcClient>(client: &C, address: &str, count: u64) -> LabResult<Vec<String>> {
    let raw = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), address.to_owned()],
    )?;
    let value = parse_cli_value(&raw)?;
    let arr = value
        .as_array()
        .ok_or_else(|| LabError::Parse("Expected array of block hashes".to_owned()))?;
    let hashes = arr
        .iter()
        .map(|v| {
            v.as_str()
                .map(String::from)
                .ok_or_else(|| LabError::Parse("Expected block hash to be string".to_owned()))
        })
        .collect::<LabResult<Vec<String>>>()?;
    Ok(hashes)
}

/// Read the wallet's trusted, untrusted-pending, and immature balances.
pub fn get_balances<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<WalletBalances> {
    let raw = client.call(Some(wallet_name), "getbalances", &[])?;
    let value = parse_cli_value(&raw)?;
    let mine = value
        .get("mine")
        .ok_or_else(|| LabError::MissingField("mine"))?;
    let trusted = mine
        .get("trusted")
        .and_then(Value::as_f64)
        .ok_or_else(|| LabError::MissingField("trusted"))?;
    let untrusted_pending = mine
        .get("untrusted_pending")
        .and_then(Value::as_f64)
        .ok_or_else(|| LabError::MissingField("untrusted_pending"))?;
    let immature = mine
        .get("immature")
        .and_then(Value::as_f64)
        .ok_or_else(|| LabError::MissingField("immature"))?;
    Ok(WalletBalances {
        trusted,
        untrusted_pending,
        immature,
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
        &[address.to_owned(), amount_btc.to_string()],
    )?;
    let value = parse_cli_value(&raw)?;
    let txid = value
        .as_str()
        .ok_or_else(|| LabError::Parse("Expected TXID to be string".to_owned()))?;
    Ok(txid.to_owned())
}

/// Mine one block, prove the reward is immature, then mine 100 more blocks.
pub fn demonstrate_coinbase_maturity<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    miner_address: &str,
    receiver_address: &str,
) -> LabResult<CoinbaseMaturityReport> {
    // 1. Mine one block.
    mine_blocks(client, miner_address, 1)?;

    // 2. Record height and balances.
    let raw_height = client.call(None, "getblockcount", &[])?;
    let value_height = parse_cli_value(&raw_height)?;
    let height_after_first_block = value_height
        .as_u64()
        .ok_or_else(|| LabError::Parse("Expected integer height".to_owned()))?;
    let balance_after_first_block = get_balances(client, miner_wallet)?;

    // 3. Attempt a 1 BTC payment and capture its error text.
    let payment_res = attempt_payment(client, miner_wallet, receiver_address, 1.0);
    let premature_spend_error = match payment_res {
        Ok(_) => {
            return Err(LabError::Rpc(
                "Expected Insufficient funds error".to_owned(),
            ))
        }
        Err(LabError::Rpc(msg)) => msg,
        Err(other) => return Err(other),
    };

    // 4. Mine 100 more blocks.
    mine_blocks(client, miner_address, 100)?;

    // 5. Record final height and balances.
    let raw_final_height = client.call(None, "getblockcount", &[])?;
    let value_final_height = parse_cli_value(&raw_final_height)?;
    let final_height = value_final_height
        .as_u64()
        .ok_or_else(|| LabError::Parse("Expected integer height".to_owned()))?;
    let final_balance = get_balances(client, miner_wallet)?;

    Ok(CoinbaseMaturityReport {
        height_after_first_block,
        balance_after_first_block,
        premature_spend_error,
        final_height,
        final_balance,
    })
}
