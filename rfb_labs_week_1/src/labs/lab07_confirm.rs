//! Lab 07 — confirm a transaction and prove block membership.

use crate::model::ConfirmationReport;
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    let raw = client.call(
        None,
        "generatetoaddress",
        &["1".to_owned(), miner_address.to_owned()],
    )?;

    let value = parse_cli_value(&raw)?;

    value
        .as_array()
        .and_then(|blocks| blocks.first())
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or(LabError::MissingField("block hash"))
}

/// Return true if the mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    let raw = client.call(None, "getrawmempool", &[])?;

    let value = parse_cli_value(&raw)?;

    Ok(value
        .as_array()
        .map(|transactions| transactions.is_empty())
        .unwrap_or(false))
}

/// Read the confirmation count for a wallet transaction.
pub fn transaction_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let raw = client.call(
        Some(wallet_name),
        "gettransaction",
        &[txid.to_owned()],
    )?;

    let value = parse_cli_value(&raw)?;

    value
        .get("confirmations")
        .and_then(Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))
}

/// Confirm a transaction and verify that it exists inside its confirming block.
pub fn confirm_and_locate_transaction<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
    miner_address: &str,
) -> LabResult<ConfirmationReport> {
    mine_one_block(client, miner_address)?;

    let mempool_is_empty = mempool_is_empty(client)?;

    let raw = client.call(
        Some(wallet_name),
        "gettransaction",
        &[txid.to_owned()],
    )?;

    let transaction = parse_cli_value(&raw)?;

    let block_hash = transaction
        .get("blockhash")
        .and_then(Value::as_str)
        .ok_or(LabError::MissingField("blockhash"))?
        .to_owned();

    let confirmations = transaction
        .get("confirmations")
        .and_then(Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))?;

    // Bitcoin Core getblock RPC call using only the block hash.
    let block_raw = client.call(None, "getblock", &[block_hash.clone()])?;

    let block = parse_cli_value(&block_raw)?;

    let transaction_is_in_block = block
        .get("tx")
        .and_then(Value::as_array)
        .map(|transactions| {
            transactions
                .iter()
                .any(|entry| entry.as_str() == Some(txid))
        })
        .unwrap_or(false);

    Ok(ConfirmationReport {
        txid: txid.to_owned(),
        block_hash,
        confirmations,
        mempool_is_empty,
        transaction_is_in_block,
    })
}