//! Lab 07 — confirm a transaction and prove block membership.

use serde_json::Value;

use crate::model::ConfirmationReport;
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    let raw = client.call(
        None,
        "generatetoaddress",
        &["1".to_string(), miner_address.to_string()],
    )?;
    let value = parse_cli_value(&raw)?;

    match value {
        Value::Array(mut items) => items
            .pop()
            .and_then(|item| item.as_str().map(ToOwned::to_owned))
            .ok_or(LabError::Parse("expected a block hash".to_owned())),
        other => Err(LabError::Parse(format!("expected array, got {other}"))),
    }
}

/// Return true only when this node's mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    let raw = client.call(None, "getrawmempool", &[])?;
    let value = parse_cli_value(&raw)?;

    match value {
        Value::Array(items) => Ok(items.is_empty()),
        other => Err(LabError::Parse(format!("expected array, got {other}"))),
    }
}

/// Return a transaction's confirmation count in the selected wallet.
pub fn transaction_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let value = parse_cli_value(&raw)?;

    value
        .get("confirmations")
        .and_then(Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))
}

/// Mine, locate the transaction's block, and prove that the block contains the TXID.
pub fn confirm_and_locate_transaction<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
    miner_address: &str,
) -> LabResult<ConfirmationReport> {
    let _block_hash = mine_one_block(client, miner_address)?;
    let mempool_is_empty = mempool_is_empty(client)?;

    let raw_tx = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let tx_value = parse_cli_value(&raw_tx)?;
    let confirmations = tx_value
        .get("confirmations")
        .and_then(Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))?;
    let block_hash = tx_value
        .get("blockhash")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or(LabError::MissingField("blockhash"))?;

    let raw_block = client.call(None, "getblock", &[block_hash.clone(), "1".to_string()])?;
    let block_value = parse_cli_value(&raw_block)?;
    let tx_list = block_value
        .get("tx")
        .and_then(Value::as_array)
        .ok_or(LabError::MissingField("tx"))?;
    let transaction_is_in_block = tx_list.iter().any(|entry| entry.as_str() == Some(txid));

    Ok(ConfirmationReport {
        txid: txid.to_owned(),
        block_hash,
        confirmations,
        mempool_is_empty,
        transaction_is_in_block,
    })
}
