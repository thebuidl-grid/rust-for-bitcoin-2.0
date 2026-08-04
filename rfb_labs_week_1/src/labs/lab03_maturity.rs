//! Lab 03 — demonstrate coinbase maturity.

use crate::labs::lab01_network::get_block_height;
use crate::model::{CoinbaseMaturityReport, WalletBalances};
use crate::rpc::RpcClient;
use crate::{LabError, LabResult};
use serde_json::Value;

/// Mine `count` blocks to an address and return the generated block hashes.
pub fn mine_blocks<C: RpcClient>(client: &C, address: &str, count: u64) -> LabResult<Vec<String>> {
    let raw = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), address.to_string()],
    )?;
    let value: Value = serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    value
        .as_array()
        .ok_or(LabError::MissingField("generatetoaddress"))?
        .iter()
        .map(|hash| hash.as_str().map(ToOwned::to_owned))
        .collect::<Option<Vec<String>>>()
        .ok_or(LabError::MissingField("generatetoaddress"))
}

/// Read the wallet's trusted, untrusted-pending, and immature balances.
pub fn get_balances<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<WalletBalances> {
    let raw = client.call(Some(wallet_name), "getbalances", &[])?;
    let value: Value = serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;
    let mine = value.get("mine").ok_or(LabError::MissingField("mine"))?;

    Ok(WalletBalances {
        trusted: mine["trusted"]
            .as_f64()
            .ok_or(LabError::MissingField("trusted"))?,
        untrusted_pending: mine["untrusted_pending"]
            .as_f64()
            .ok_or(LabError::MissingField("untrusted_pending"))?,
        immature: mine["immature"]
            .as_f64()
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
        &[address.to_string(), format!("{amount_btc}")],
    )?;
    Ok(raw.trim().trim_matches('"').to_string())
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
        Err(LabError::Rpc(message)) => message,
        Err(other) => return Err(other),
        Ok(txid) => {
            return Err(LabError::Rpc(format!(
                "expected premature spend to fail, but it succeeded with txid {txid}"
            )))
        }
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
