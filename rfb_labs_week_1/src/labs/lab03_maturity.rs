//! Lab 03 — demonstrate coinbase maturity.

use crate::model::{CoinbaseMaturityReport, WalletBalances};
use crate::rpc::parse_cli_value;
use crate::rpc::RpcClient;
use crate::LabError;
use crate::LabResult;

/// Mine `count` blocks to an address and return the generated block hashes.
pub fn mine_blocks<C: RpcClient>(client: &C, address: &str, count: u64) -> LabResult<Vec<String>> {
    // TODO: call generatetoaddress with count and address.
    let call = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), address.to_string()],
    )?;
    let val = parse_cli_value(&call)?;

    serde_json::from_value::<Vec<String>>(val).map_err(Into::into)
}

/// Read the wallet's trusted, untrusted-pending, and immature balances.
pub fn get_balances<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<WalletBalances> {
    // TODO: call getbalances in wallet context and decode the nested `mine` object.
    let call = client.call(Some(wallet_name), "getbalances", &[])?;
    let val = parse_cli_value(&call)?;

    let mine = val
        .get("mine")
        .ok_or_else(|| LabError::Parse("getbalances response missing mine".to_string()))?;

    serde_json::from_value::<WalletBalances>(mine.clone()).map_err(Into::into)
}

/// Attempt a wallet payment and return either its TXID or the Bitcoin Core error.
pub fn attempt_payment<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
    amount_btc: f64,
) -> LabResult<String> {
    // TODO: call sendtoaddress. Do not hide an insufficient-funds RPC error.
    let call = client.call(
        Some(wallet_name),
        "sendtoaddress",
        &[address.to_string(), amount_btc.to_string()],
    )?;
    let val = parse_cli_value(&call)?;

    val.as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| LabError::Parse("sendtoaddress response is not a string".to_string()))
}

pub fn demonstrate_coinbase_maturity<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    miner_address: &str,
    receiver_address: &str,
) -> LabResult<CoinbaseMaturityReport> {
    // 1. Mine one block.
    mine_blocks(client, miner_address, 1)?;

    // 2. Record height and balances.
    let call = client.call(None, "getblockcount", &[])?;
    let val = parse_cli_value(&call)?;
    let height_after_first_block = val
        .as_u64()
        .ok_or_else(|| LabError::Parse("getblockcount response is not a u64".to_string()))?;

    let balance_after_first_block = get_balances(client, miner_wallet)?;

    // 3. Attempt a 1 BTC payment and capture its error text (don't propagate — this failure is expected).
    let premature_spend_error = match attempt_payment(client, miner_wallet, receiver_address, 1.0) {
        Ok(txid) => format!("unexpected success: {txid}"),
        Err(LabError::Rpc(msg)) => msg,
        Err(e) => e.to_string(),
    };

    // 4. Mine 100 more blocks.
    mine_blocks(client, miner_address, 100)?;

    // 5. Record final height and balances.
    let call = client.call(None, "getblockcount", &[])?;
    let val = parse_cli_value(&call)?;
    let final_height = val
        .as_u64()
        .ok_or_else(|| LabError::Parse("getblockcount response is not a u64".to_string()))?;

    let final_balance = get_balances(client, miner_wallet)?;

    Ok(CoinbaseMaturityReport {
        height_after_first_block,
        balance_after_first_block,
        premature_spend_error,
        final_height,
        final_balance,
    })
}
