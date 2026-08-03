//! Lab 03 — demonstrate coinbase maturity.

use crate::labs::lab01_network::get_block_height;
use crate::model::{CoinbaseMaturityReport, WalletBalances};
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

/// Mine `count` blocks to an address and return the generated block hashes.
pub fn mine_blocks<C: RpcClient>(client: &C, address: &str, count: u64) -> LabResult<Vec<String>> {
    let raw = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), address.to_owned()],
    )?;
    Ok(serde_json::from_str(&raw)?)
}

/// Read the wallet's trusted, untrusted-pending, and immature balances.
pub fn get_balances<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<WalletBalances> {
    let raw = client.call(Some(wallet_name), "getbalances", &[])?;
    let value = parse_cli_value(&raw)?;
    let mine = value.get("mine").ok_or(LabError::MissingField("mine"))?;
    Ok(serde_json::from_value(mine.clone())?)
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
        &[address.to_owned(), amount_btc.to_string()],
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
    let height_after_first_block = get_block_height(client)?;
    let balance_after_first_block = get_balances(client, miner_wallet)?;

    let premature_spend_error = match attempt_payment(client, miner_wallet, receiver_address, 1.0) {
        Ok(txid) => {
            return Err(LabError::Rpc(format!(
                "expected the premature payment to fail, but it succeeded as `{txid}`"
            )))
        }
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
