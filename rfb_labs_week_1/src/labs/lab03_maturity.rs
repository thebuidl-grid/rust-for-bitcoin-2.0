//! Lab 03 — demonstrate coinbase maturity.

use crate::labs::lab01_network::{self, get_block_height};
use crate::labs::lab02_wallets;
use crate::model::{CoinbaseMaturityReport, WalletBalances};
use crate::rpc::{parse_cli_value, required_f64, RpcClient};
use crate::{LabError, LabResult};

/// Mine `count` blocks to an address and return the generated block hashes.
pub fn mine_blocks<C: RpcClient>(client: &C, address: &str, count: u64) -> LabResult<Vec<String>> {
    let raw = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), address.to_string()],
    )?;
    let value = parse_cli_value(&raw)?;
    let arr = value.as_array().ok_or(LabError::Parse(
        "Expected JSON array from generatetoaddress".to_string(),
    ))?;
    let block_hashes = arr
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_string)
                .ok_or(LabError::Parse("expected string block hash".to_string()))
        })
        .collect::<LabResult<Vec<_>>>()?;

    Ok(block_hashes)
}

/// Read the wallet's trusted, untrusted-pending, and immature balances.
pub fn get_balances<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<WalletBalances> {
    let raw = client.call(Some(wallet_name), "getbalances", &[])?;
    let value = parse_cli_value(&raw)?;
    let mine = value.get("mine").ok_or(LabError::MissingField("mine"))?;

    let trusted = required_f64(mine, "trusted")?;
    let untrusted_pending = required_f64(mine, "untrusted_pending")?;
    let immature = required_f64(mine, "immature")?;

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
    // 1. Mine one block.
    mine_blocks(client, &miner_address, 1)?;
    // 2. Record height and balances.
    let height_after_first_block = get_block_height(client)?;
    let balance_after_first_block = get_balances(client, miner_wallet)?;
    // 3. Attempt a 1 BTC payment and capture its error text.
    let premature_spend_error = match attempt_payment(client, miner_wallet, receiver_address, 1.0) {
        Ok(_) => String::from("Payment succeeded unexpectedly"),
        Err(LabError::Rpc(msg)) => msg,
        Err(e) => e.to_string(),
    };

    // 4. Mine 100 more blocks.
    mine_blocks(client, &miner_address, 100)?;

    // 5. Record final height and balances.
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
