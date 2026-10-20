//! Lab 03 — demonstrate coinbase maturity.

use crate::model::{CoinbaseMaturityReport, WalletBalances};
use crate::rpc::RpcClient;
use crate::{LabResult, LabError};
use serde_json::Value;

/// Mine `count` blocks to an address and return the generated block hashes.
pub fn mine_blocks<C: RpcClient>(client: &C, address: &str, count: u64) -> LabResult<Vec<String>> {
    // TODO: call generatetoaddress with count and address.
    // todo!("Lab 03: mine blocks")
    let raw = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), address.to_string()],
    )?;

    let value: Value = serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    let hashes = value
        .as_array()
        .ok_or(LabError::MissingField("generatetoaddress"))?
        .iter()
        .map(|h| h.as_str().map(|s| s.to_string()))
        .collect::<Option<Vec<String>>>()
        .ok_or(LabError::MissingField("generatetoaddress"))?;

    Ok(hashes)
}

/// Read the wallet's trusted, untrusted-pending, and immature balances.
pub fn get_balances<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<WalletBalances> {
    // TODO: call getbalances in wallet context and decode the nested `mine` object.
    // todo!("Lab 03: inspect wallet balances")
    let raw = client.call(Some(wallet_name), "getbalances", &[])?;
    let value: Value = serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    let mine = &value["mine"];

    let trusted = mine["trusted"]
        .as_f64()
        .ok_or(LabError::MissingField("mine.trusted"))?;
    let untrusted_pending = mine["untrusted_pending"]
        .as_f64()
        .ok_or(LabError::MissingField("mine.untrusted_pending"))?;
    let immature = mine["immature"]
        .as_f64()
        .ok_or(LabError::MissingField("mine.immature"))?;

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
    // TODO: call sendtoaddress. Do not hide an insufficient-funds RPC error.
    // todo!("Lab 03: attempt a payment")
    let raw = client.call(
        Some(wallet_name),
        "sendtoaddress",
        &[address.to_string(), amount_btc.to_string()],
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
    // TODO:
    // 1. Mine one block.
    // 2. Record height and balances.
    // 3. Attempt a 1 BTC payment and capture its error text.
    // 4. Mine 100 more blocks.
    // 5. Record final height and balances.
    // todo!("Lab 03: produce coinbase-maturity evidence")
    // 1. Mine one block.
    mine_blocks(client, miner_address, 1)?;

    // 2. Record height and balances.
    let raw_height = client.call(None, "getblockcount", &[])?;
    let height_after_first_block: u64 = raw_height
        .trim()
        .parse()
        .map_err(|_| LabError::Parse(format!("invalid block height: '{}'", raw_height.trim())))?;

    let balance_after_first_block = get_balances(client, miner_wallet)?;

    // 3. Attempt a 1 BTC payment and capture its error text (coinbase is immature, spend should fail).
    let premature_spend_error = match attempt_payment(client, miner_wallet, receiver_address, 1.0) {
        Ok(txid) => txid,
        Err(LabError::Rpc(msg)) => msg,
        Err(other) => return Err(other),
    };

    // 4. Mine 100 more blocks.
    mine_blocks(client, miner_address, 100)?;

    // 5. Record final height and balances.
    let raw_final_height = client.call(None, "getblockcount", &[])?;
    let final_height: u64 = raw_final_height.trim().parse().map_err(|_| {
        LabError::Parse(format!(
            "invalid block height: '{}'",
            raw_final_height.trim()
        ))
    })?;

    let final_balance = get_balances(client, miner_wallet)?;

    Ok(CoinbaseMaturityReport {
        height_after_first_block,
        balance_after_first_block,
        premature_spend_error,
        final_height,
        final_balance,
    })
}
