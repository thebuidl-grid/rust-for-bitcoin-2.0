//! Lab 07 — confirm a transaction and prove block membership.

use crate::labs::lab05_mempool::get_raw_mempool;
use crate::model::ConfirmationReport;
use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::LabError;
use crate::LabResult;

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    let raw = client.call(
        None,
        "generatetoaddress",
        &["1".to_owned(), miner_address.to_owned()],
    )?;
    let value = parse_cli_value(&raw)?;
    let hashes = value
        .as_array()
        .ok_or_else(|| LabError::Parse("generatetoaddress did not return an array".to_owned()))?;
    let first = hashes
        .first()
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| LabError::Parse("generatetoaddress returned an empty array".to_owned()))?;
    Ok(first.to_owned())
}

/// Return true only when this node's mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    Ok(get_raw_mempool(client)?.is_empty())
}

/// Return a transaction's confirmation count in the selected wallet.
pub fn transaction_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let value = parse_cli_value(&raw)?;
    value
        .get("confirmations")
        .and_then(serde_json::Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))
}

/// Mine, locate the transaction's block, and prove that the block contains the TXID.
pub fn confirm_and_locate_transaction<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
    miner_address: &str,
) -> LabResult<ConfirmationReport> {
    mine_one_block(client, miner_address)?;
    let mempool_is_empty = mempool_is_empty(client)?;

    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let transaction = parse_cli_value(&raw)?;
    let confirmations = transaction
        .get("confirmations")
        .and_then(serde_json::Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))?;
    let block_hash = transaction
        .get("blockhash")
        .and_then(serde_json::Value::as_str)
        .ok_or(LabError::MissingField("blockhash"))?
        .to_owned();

    let raw = client.call(None, "getblock", &[block_hash.clone(), "1".to_owned()])?;
    let value = parse_cli_value(&raw)?;
    let transactions = value
        .get("tx")
        .and_then(serde_json::Value::as_array)
        .ok_or(LabError::MissingField("tx"))?;
    let transaction_is_in_block = transactions
        .iter()
        .any(|entry| entry.as_str() == Some(txid.as_ref()));

    Ok(ConfirmationReport {
        txid: txid.to_owned(),
        block_hash: required_string(&value, "hash").unwrap_or(block_hash),
        confirmations,
        mempool_is_empty,
        transaction_is_in_block,
    })
}
