//! Lab 03 — demonstrate coinbase maturity.

use crate::labs::lab01_network::get_block_height;
use crate::model::{CoinbaseMaturityReport, WalletBalances};
use crate::rpc::{parse_cli_value, required_f64, RpcClient};
use crate::{LabError, LabResult};

/// Confirmations a coinbase output must accumulate before the wallet may spend it.
pub const COINBASE_MATURITY: u64 = 100;

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
    let balances = parse_cli_value(&raw)?;
    let mine = balances.get("mine").ok_or(LabError::MissingField("mine"))?;

    Ok(WalletBalances {
        trusted: required_f64(mine, "trusted")?,
        untrusted_pending: required_f64(mine, "untrusted_pending")?,
        immature: required_f64(mine, "immature")?,
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
        &[address.to_owned(), format!("{amount_btc}")],
    )?;
    let txid = parse_cli_value(&raw)?;
    txid.as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| LabError::Parse(format!("sendtoaddress returned a non-string: {raw}")))
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

    // The reward from block 1 is immature, so Bitcoin Core must refuse this spend.
    // The refusal is the evidence, so keep its text instead of aborting.
    let premature_spend_error = match attempt_payment(client, miner_wallet, receiver_address, 1.0) {
        Err(LabError::Rpc(message)) => message,
        Ok(txid) => {
            return Err(LabError::Parse(format!(
                "expected an immature-funds refusal, but the spend succeeded as {txid}"
            )))
        }
        Err(transport_failure) => return Err(transport_failure),
    };

    mine_blocks(client, miner_address, COINBASE_MATURITY)?;

    Ok(CoinbaseMaturityReport {
        height_after_first_block,
        balance_after_first_block,
        premature_spend_error,
        final_height: get_block_height(client)?,
        final_balance: get_balances(client, miner_wallet)?,
    })
}
