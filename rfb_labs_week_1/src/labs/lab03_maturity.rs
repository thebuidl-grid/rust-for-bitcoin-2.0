//! Lab 03 — demonstrate coinbase maturity.

use crate::model::{CoinbaseMaturityReport, WalletBalances};
use crate::rpc::{required_f64, RpcClient};
use crate::{LabError, LabResult};

/// Mine `count` blocks to an address and return the generated block hashes.
pub fn mine_blocks<C: RpcClient>(client: &C, address: &str, count: u64) -> LabResult<Vec<String>> {
    let response = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), address.to_string()],
    )?;
    let block_hashes = serde_json::from_str::<Vec<String>>(&response)?;

    Ok(block_hashes)
}

/// Read the wallet's trusted, untrusted-pending, and immature balances.
pub fn get_balances<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<WalletBalances> {
    let response = client.call(Some(wallet_name), "getbalances", &[])?;
    let balances: serde_json::Value = serde_json::from_str(&response)?;
    let mine = balances.get("mine").ok_or(LabError::MissingField("mine"))?;

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
    mine_blocks(client, miner_address, 1)?;
    let height_response = client.call(None, "getblockcount", &[])?;
    let height_after_first_block = height_response
        .parse::<u64>()
        .map_err(|error| LabError::Parse(error.to_string()))?;
    let balance_after_first_block = get_balances(client, miner_wallet)?;
    let premature_spend_error = match attempt_payment(client, miner_wallet, receiver_address, 1.0) {
        Err(LabError::Rpc(error)) => error,
        Err(error) => return Err(error),
        Ok(_) => {
            return Err(LabError::Parse(
                "premature coinbase spend unexpectedly succeeded".to_string(),
            ))
        }
    };

    mine_blocks(client, miner_address, 100)?;
    let height_response = client.call(None, "getblockcount", &[])?;
    let final_height = height_response
        .parse::<u64>()
        .map_err(|error| LabError::Parse(error.to_string()))?;
    let final_balance = get_balances(client, miner_wallet)?;

    Ok(CoinbaseMaturityReport {
        height_after_first_block,
        balance_after_first_block,
        premature_spend_error,
        final_height,
        final_balance,
    })
}
