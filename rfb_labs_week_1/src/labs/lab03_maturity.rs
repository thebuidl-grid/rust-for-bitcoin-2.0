//! Lab 03 — demonstrate coinbase maturity.

use crate::model::{CoinbaseMaturityReport, WalletBalances};
use crate::rpc::{parse_cli_value, required_f64, required_u64, RpcClient};
use crate::{LabError, LabResult};

/// Mine `count` blocks to an address and return the generated block hashes.
pub fn mine_blocks<C: RpcClient>(client: &C, address: &str, count: u64) -> LabResult<Vec<String>> {
    // TODO: call generatetoaddress with count and address.
    let raw = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), address.to_string()],
    )?;
    let val = parse_cli_value(&raw)?;

    val.as_array()
        .ok_or_else(|| LabError::Parse("Expected array of block hashes".into()))?
        .iter()
        .map(|item| {
            item.as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| LabError::Parse("Invalid block hash string".into()))
        })
        .collect()
}

/// Read the wallet's trusted, untrusted-pending, and immature balances.
pub fn get_balances<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<WalletBalances> {
    // TODO: call getbalances in wallet context and decode the nested `mine` object.
     let raw = client.call(Some(wallet_name), "getbalances", &[])?;
    let val = parse_cli_value(&raw)?;

    let mine_obj = val
        .get("mine")
        .ok_or_else(|| LabError::MissingField("mine"))?;

    let trusted = required_f64(mine_obj, "trusted")?;
    let untrusted_pending = required_f64(mine_obj, "untrusted_pending")?;
    let immature = required_f64(mine_obj, "immature")?;

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
    let amount_str = if amount_btc.fract() == 0.0 {
        (amount_btc as u64).to_string()
    } else {
        amount_btc.to_string()
    };

    let raw = client.call(
        Some(wallet_name),
        "sendtoaddress",
        &[address.to_string(), amount_str],
    )?;
    let val = parse_cli_value(&raw)?;

    val.as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| LabError::Parse("Expected TXID string".into()))
}

/// Mine one block, prove the reward is immature, then mine 100 more blocks.
pub fn demonstrate_coinbase_maturity<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    miner_address: &str,
    receiver_address: &str,
) -> LabResult<CoinbaseMaturityReport> {
 mine_blocks(client, miner_address, 1)?;

    // 2. Get height & balances after first block
    let height_raw = client.call(None, "getblockcount", &[])?;
    let height_val = parse_cli_value(&height_raw)?;
    let height_after_first_block = if let Some(h) = height_val.as_u64() {
        h
    } else {
        required_u64(&height_val, "getblockcount")?
    };

    let balance_after_first_block = get_balances(client, miner_wallet)?;

    // 3. Attempt payment and capture error text explicitly
    let premature_spend_error = match attempt_payment(client, miner_wallet, receiver_address, 1.0) {
        Ok(txid) => txid,
        Err(LabError::Rpc(msg)) => msg,
        Err(e) => return Err(e),
    };

    // 4. Mine 100 more blocks to reach maturity
    mine_blocks(client, miner_address, 100)?;

    // 5. Get final height & balances
    let final_height_raw = client.call(None, "getblockcount", &[])?;
    let final_height_val = parse_cli_value(&final_height_raw)?;
    let final_height = if let Some(h) = final_height_val.as_u64() {
        h
    } else {
        required_u64(&final_height_val, "getblockcount")?
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
