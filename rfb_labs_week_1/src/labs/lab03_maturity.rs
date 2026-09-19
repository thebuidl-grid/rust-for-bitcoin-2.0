//! Lab 03 — mine blocks and observe coinbase maturity.

use crate::model::{CoinbaseMaturityReport, WalletBalances};
use crate::rpc::RpcClient;
use crate::{LabError, LabResult};
use serde::Deserialize;

#[derive(Deserialize)]
struct GetBalancesResponse {
    mine: WalletBalances,
}

/// Query wallet balances from `getbalances`.
pub fn get_balances<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<WalletBalances> {
    let json_str = client.call(Some(wallet_name), "getbalances", &[])?;
    let resp: GetBalancesResponse = serde_json::from_str(&json_str)?;
    Ok(resp.mine)
}

/// Mine a specified number of blocks to an address in global RPC scope (None).
pub fn mine_blocks<C: RpcClient>(
    client: &C,
    address: &str,
    count: u64,
) -> LabResult<Vec<String>> {
    let json_str = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), address.to_string()],
    )?;
    let block_hashes: Vec<String> = serde_json::from_str(&json_str)?;
    Ok(block_hashes)
}

/// Query current blockchain height via `getblockcount`.
pub fn get_block_count<C: RpcClient>(client: &C) -> LabResult<u64> {
    let json_str = client.call(None, "getblockcount", &[])?;
    let count: u64 = serde_json::from_str(&json_str)?;
    Ok(count)
}

/// Attempt to send payment within a given wallet context.
pub fn attempt_payment<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    receiver_address: &str,
    amount: f64,
) -> LabResult<String> {
    client.call(
        Some(wallet_name),
        "sendtoaddress",
        &[receiver_address.to_string(), amount.to_string()],
    )
}

/// Demonstrate coinbase maturity: mine 1 block, attempt premature spend, and mine up to height 101.
pub fn demonstrate_coinbase_maturity<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    miner_address: &str,
    receiver_address: &str,
) -> LabResult<CoinbaseMaturityReport> {
    // 1. Mine 1 block (global scope) & get height
    mine_blocks(client, miner_address, 1)?;
    let height_after_first_block = get_block_count(client)?;
    let balance_after_first_block = get_balances(client, wallet_name)?;

    // 2. Attempt premature spend and capture the RPC error string
    let premature_spend_error = match attempt_payment(client, wallet_name, receiver_address, 1.0) {
        Ok(txid) => txid,
        Err(LabError::Rpc(msg)) => msg,
        Err(e) => return Err(e),
    };

    // 3. Mine 100 additional blocks (global scope) & get final height
    mine_blocks(client, miner_address, 100)?;
    let final_height = get_block_count(client)?;
    let final_balance = get_balances(client, wallet_name)?;

    Ok(CoinbaseMaturityReport {
        height_after_first_block,
        balance_after_first_block,
        premature_spend_error,
        final_height,
        final_balance,
    })
}