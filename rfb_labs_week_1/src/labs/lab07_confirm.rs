//! Lab 07 — confirm a transaction and prove block membership.

use crate::model::ConfirmationReport;
use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

/// `getblock` verbosity that returns the header plus the block's TXID list.
const TXID_LIST_VERBOSITY: &str = "1";

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    let raw = client.call(
        None,
        "generatetoaddress",
        &["1".to_owned(), miner_address.to_owned()],
    )?;

    string_array(&parse_cli_value(&raw)?, "generatetoaddress")?
        .into_iter()
        .next()
        .ok_or_else(|| LabError::Parse("generatetoaddress produced no block".to_owned()))
}

/// Return true only when this node's mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    let raw = client.call(None, "getrawmempool", &[])?;
    Ok(string_array(&parse_cli_value(&raw)?, "getrawmempool")?.is_empty())
}

/// Return a transaction's confirmation count in the selected wallet.
pub fn transaction_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    read_confirmations(&parse_cli_value(&raw)?)
}

/// Mine, locate the transaction's block, and prove that the block contains the TXID.
pub fn confirm_and_locate_transaction<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
    miner_address: &str,
) -> LabResult<ConfirmationReport> {
    mine_one_block(client, miner_address)?;

    let mempool_cleared = mempool_is_empty(client)?;

    // One `gettransaction` call carries both facts the report needs, so the depth and
    // the containing block hash are guaranteed to describe the same observation.
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let transaction = parse_cli_value(&raw)?;
    let block_hash = required_string(&transaction, "blockhash")?;
    let confirmations = read_confirmations(&transaction)?;

    // Ask the block itself rather than trusting the wallet: the TXID has to appear in
    // the block's own transaction list.
    let raw = client.call(
        None,
        "getblock",
        &[block_hash.clone(), TXID_LIST_VERBOSITY.to_owned()],
    )?;
    let block = parse_cli_value(&raw)?;
    let block_txids = block
        .get("tx")
        .ok_or(LabError::MissingField("tx"))
        .and_then(|value| string_array(value, "getblock"))?;

    Ok(ConfirmationReport {
        txid: txid.to_owned(),
        block_hash,
        confirmations,
        mempool_is_empty: mempool_cleared,
        transaction_is_in_block: block_txids.iter().any(|entry| entry == txid),
    })
}

/// Read a signed confirmation count from a `gettransaction` response.
fn read_confirmations(value: &Value) -> LabResult<i64> {
    value
        .get("confirmations")
        .and_then(Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))
}

/// Decode a JSON array of strings such as a block's `tx` list.
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
