//! Lab 03 — demonstrate coinbase maturity.

use crate::model::{CoinbaseMaturityReport, WalletBalances};
use crate::rpc::RpcClient;
use crate::LabError;
use crate::LabResult;

/// Mine `count` blocks to an address and return the generated block hashes.
pub fn mine_blocks<C: RpcClient>(client: &C, address: &str, count: u64) -> LabResult<Vec<String>> {
    let raw = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), address.to_owned()],
    )?;
    let value = crate::rpc::parse_cli_value(&raw)?;
    Ok(serde_json::from_value(value)?)
}

/// Read the wallet's trusted, untrusted-pending, and immature balances.
pub fn get_balances<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<WalletBalances> {
    // TODO: call getbalances in wallet context and decode the nested `mine` object.
    let raw = client.call(Some(wallet_name), "getbalances", &[])?;
    let value = crate::rpc::parse_cli_value(&raw)?;
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
    // TODO: call sendtoaddress. Do not hide an insufficient-funds RPC error.
    let raw = client.call(
        Some(wallet_name),
        "sendtoaddress",
        &[address.to_owned(), amount_btc.to_string()],
    )?;
    let value = crate::rpc::parse_cli_value(&raw)?;
    value
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| LabError::Parse(format!("expected a string, got: {value}")))
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

    mine_blocks(client, miner_address, 1)?;
    let height_after_first_block = read_block_height(client)?;
    let balance_after_first_block = get_balances(client, miner_wallet)?;

    let premature_spend_error = match attempt_payment(client, miner_wallet, receiver_address, 1.0) {
        Err(LabError::Rpc(err)) => err,
        Err(other) => return Err(other),
        Ok(txid) => {
            return Err(LabError::Rpc(format!(
                "expected a premature spend to fail, but it succeeded with txid {txid}"
            )))
        }
    };

    fn read_block_height<C: RpcClient>(client: &C) -> LabResult<u64> {
        let raw = client.call(None, "getblockcount", &[])?;
        let value = crate::rpc::parse_cli_value(&raw)?;
        value
            .as_u64()
            .ok_or_else(|| LabError::Parse(format!("expected a number, got: {value}")))
    }

    mine_blocks(client, miner_address, 100)?;
    let final_height = read_block_height(client)?;
    let final_balance = get_balances(client, miner_wallet)?;

    Ok(CoinbaseMaturityReport {
        height_after_first_block,
        balance_after_first_block,
        premature_spend_error,
        final_height,
        final_balance,
    })
}
