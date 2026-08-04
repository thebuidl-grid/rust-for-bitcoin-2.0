//! Lab 03 — demonstrate coinbase maturity.

use crate::model::{CoinbaseMaturityReport, WalletBalances};
use crate::rpc::{parse_cli_value, required_f64, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

/// Blocks a coinbase output must be buried under before it can be spent.
pub const COINBASE_MATURITY: u64 = 100;

/// Mine `count` blocks to an address and return the generated block hashes.
pub fn mine_blocks<C: RpcClient>(client: &C, address: &str, count: u64) -> LabResult<Vec<String>> {
    let raw = client.call(
        None,
        "generatetoaddress",
        &[count.to_string(), address.to_owned()],
    )?;
    string_array(&parse_cli_value(&raw)?, "generatetoaddress")
}

/// Read the wallet's trusted, untrusted-pending, and immature balances.
pub fn get_balances<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<WalletBalances> {
    let raw = client.call(Some(wallet_name), "getbalances", &[])?;
    let value = parse_cli_value(&raw)?;

    // `getbalances` groups balances by ownership; the labs only care about `mine`.
    let mine = value.get("mine").ok_or(LabError::MissingField("mine"))?;

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
        &[address.to_owned(), format_amount(amount_btc)],
    )?;
    parse_cli_value(&raw)?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| LabError::Parse(format!("sendtoaddress returned `{raw}`")))
}

/// Mine one block, prove the reward is immature, then mine 100 more blocks.
pub fn demonstrate_coinbase_maturity<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    miner_address: &str,
    receiver_address: &str,
) -> LabResult<CoinbaseMaturityReport> {
    mine_blocks(client, miner_address, 1)?;

    let height_after_first_block = block_count(client)?;
    let balance_after_first_block = get_balances(client, miner_wallet)?;

    // The single coinbase reward is still immature, so this payment must fail. Keep
    // Bitcoin Core's own wording as the evidence rather than inventing a message.
    let premature_spend_error = match attempt_payment(client, miner_wallet, receiver_address, 1.0) {
        Err(LabError::Rpc(message)) => message,
        Err(other) => return Err(other),
        Ok(txid) => {
            return Err(LabError::Parse(format!(
                "the immature reward was unexpectedly spendable: {txid}"
            )))
        }
    };

    mine_blocks(client, miner_address, COINBASE_MATURITY)?;

    let final_height = block_count(client)?;
    let final_balance = get_balances(client, miner_wallet)?;

    Ok(CoinbaseMaturityReport {
        height_after_first_block,
        balance_after_first_block,
        premature_spend_error,
        final_height,
        final_balance,
    })
}

/// Read the node-wide block height.
fn block_count<C: RpcClient>(client: &C) -> LabResult<u64> {
    let raw = client.call(None, "getblockcount", &[])?;
    parse_cli_value(&raw)?
        .as_u64()
        .ok_or_else(|| LabError::Parse(format!("getblockcount returned `{raw}`")))
}

/// Render a BTC amount the way `bitcoin-cli` expects it.
///
/// Bitcoin has eight decimal places, so formatting at that precision and trimming the
/// padding keeps binary floating-point noise out of the command line.
fn format_amount(amount_btc: f64) -> String {
    let rendered = format!("{amount_btc:.8}");
    let trimmed = rendered.trim_end_matches('0').trim_end_matches('.');
    trimmed.to_owned()
}

/// Decode a JSON array of strings such as the `generatetoaddress` response.
fn string_array(value: &Value, method: &str) -> LabResult<Vec<String>> {
    value
        .as_array()
        .ok_or_else(|| LabError::Parse(format!("{method} did not return an array")))?
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| LabError::Parse(format!("{method} returned a non-string entry")))
        })
        .collect()
}
