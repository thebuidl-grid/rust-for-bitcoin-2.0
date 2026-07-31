//! Lab 03 — demonstrate coinbase maturity.

use crate::model::{CoinbaseMaturityReport, WalletBalances};
use crate::rpc::{parse_cli_value, required_f64, required_u64, RpcClient};
use crate::LabResult;

/// Mine `count` blocks to an address and return the generated block hashes.
pub fn mine_blocks<C: RpcClient>(
    client: &C,
    address: &str,
    count: u64,
) -> LabResult<Vec<String>> {
    let response = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), address.to_string()],
    )?;

    let value = parse_cli_value(&response)?;

    Ok(value
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect())
}

/// Read the wallet's trusted, untrusted-pending, and immature balances.
pub fn get_balances<C: RpcClient>(
    client: &C,
    wallet_name: &str,
) -> LabResult<WalletBalances> {
    let response = client.call(Some(wallet_name), "getbalances", &[])?;
    let value = parse_cli_value(&response)?;

    let mine = &value["mine"];

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
    client.call(
        Some(wallet_name),
        "sendtoaddress",
        &[address.to_string(), amount_btc.to_string()],
    )
}

/// Mine one block, prove the reward is immature, then mine 100 more blocks.
pub fn demonstrate_coinbase_maturity<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    miner_address: &str,
    receiver_address: &str,
) -> LabResult<CoinbaseMaturityReport> {
    mine_blocks(client, miner_address, 1)?;

    let height_after_first_block = {
        let response = client.call(None, "getblockcount", &[])?;
        let value = parse_cli_value(&response)?;
        value
            .as_u64()
            .ok_or(crate::LabError::Parse("expected block height".to_string()))?
    };

    let balance_after_first_block = get_balances(client, miner_wallet)?;

    let premature_spend_error =
        match attempt_payment(client, miner_wallet, receiver_address, 1.0) {
            Ok(txid) => txid,
            Err(crate::LabError::Rpc(message)) => message,
            Err(err) => return Err(err),
        };

    mine_blocks(client, miner_address, 100)?;

    let final_height = {
        let response = client.call(None, "getblockcount", &[])?;
        let value = parse_cli_value(&response)?;
        required_u64(&value, "")
            .or_else(|_| {
                value
                    .as_u64()
                    .ok_or(crate::LabError::Parse("expected block height".to_string()))
            })?
    };

    let final_balance = get_balances(client, miner_wallet)?;

    Ok(CoinbaseMaturityReport {
        height_after_first_block,
        balance_after_first_block,
        premature_spend_error,
        final_height,
        final_balance,
    })
}