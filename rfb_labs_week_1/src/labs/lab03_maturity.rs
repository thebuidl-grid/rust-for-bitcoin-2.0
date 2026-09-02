//! Lab 03 — demonstrate coinbase maturity.

use crate::model::{CoinbaseMaturityReport, WalletBalances};
use crate::rpc::{parse_cli_value, required_f64, RpcClient};
use crate::{LabError, LabResult};

/// Mine `count` blocks to an address and return the generated block hashes.
pub fn mine_blocks<C: RpcClient>(client: &C, address: &str, count: u64) -> LabResult<Vec<String>> {
    // 1. Call `generatetoaddress` with count and address
    let raw = client.call(None, "generatetoaddress", &[count.to_string(), address.into()])?;
    let value = parse_cli_value(&raw)?;

    // 2. Decode JSON array of block hashes
    let hashes = value
        .as_array()
        .ok_or_else(|| LabError::Parse("generatetoaddress did not return an array".to_owned()))?
        .iter()
        .filter_map(|val| val.as_str().map(ToOwned::to_owned))
        .collect();

    Ok(hashes)
}

/// Read the wallet's trusted, untrusted-pending, and immature balances.
pub fn get_balances<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<WalletBalances> {
    // 1. Call `getbalances` in the wallet context
    let raw = client.call(Some(wallet_name), "getbalances", &[])?;
    let value = parse_cli_value(&raw)?;

    // 2. Navigate to the nested `mine` object
    let mine = value
        .get("mine")
        .ok_or_else(|| LabError::Parse("getbalances response missing 'mine' object".to_owned()))?;

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
    let raw = client.call(
        Some(wallet_name),
        "sendtoaddress",
        &[address.into(), amount_btc.to_string()],
    )?;

    let value = parse_cli_value(&raw)?;
    value
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| LabError::Parse("sendtoaddress did not return a txid string".to_owned()))
}

/// Mine one block, prove the reward is immature, then mine 100 more blocks.
pub fn demonstrate_coinbase_maturity<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    miner_address: &str,
    receiver_address: &str,
) -> LabResult<CoinbaseMaturityReport> {
    // 1. Mine 1 block to miner_address
    let _ = mine_blocks(client, miner_address, 1)?;

    // Read current height
    let raw_height = client.call(None, "getblockcount", &[])?;
    let val_height = parse_cli_value(&raw_height)?;
    let initial_height = val_height
        .as_u64()
        .ok_or_else(|| LabError::Parse("getblockcount did not return a u64".to_owned()))?;

    // 2. Record initial balances (should show immature = 50.0 BTC)
    let initial_balances = get_balances(client, miner_wallet)?;

    // 3. Attempt payment of 1 BTC (will fail with insufficient funds because 50 BTC reward is immature)
    let failed_payment_error = match attempt_payment(client, miner_wallet, receiver_address, 1.0) {
        Ok(txid) => {
            return Err(LabError::Parse(format!(
                "expected sendtoaddress to fail while coinbase is immature, but it succeeded with txid {txid}"
            )))
        }
        Err(LabError::Rpc(message)) => message,
        Err(other) => return Err(other),
    };

    // 4. Mine 100 more blocks to reach maturity (100 total confirmations required for coinbase)
    let _ = mine_blocks(client, miner_address, 100)?;

    // Read final height
    let raw_final_height = client.call(None, "getblockcount", &[])?;
    let val_final_height = parse_cli_value(&raw_final_height)?;
    let matured_height = val_final_height
        .as_u64()
        .ok_or_else(|| LabError::Parse("getblockcount did not return a u64".to_owned()))?;

    // 5. Record final balances (trusted balance now spendable)
    let matured_balances = get_balances(client, miner_wallet)?;

    Ok(CoinbaseMaturityReport {
        height_after_first_block: initial_height,
        balance_after_first_block: initial_balances,
        premature_spend_error: failed_payment_error,
        final_height: matured_height,
        final_balance: matured_balances,
    })
}