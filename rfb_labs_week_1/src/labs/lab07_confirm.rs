//! Lab 07 — confirm a transaction and prove block membership.

use serde_json::Value;

use crate::model::ConfirmationReport;
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

fn fetch_transaction<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<(i64, Option<String>)> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let value = parse_cli_value(&raw)?;
    let confirmations = value
        .get("confirmations")
        .and_then(Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))?;
    let block_hash = value
        .get("blockhash")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    Ok((confirmations, block_hash))
}

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    // TODO: call generatetoaddress with a count of one.
    let raw = client.call(
        None,
        "generatetoaddress",
        &["1".to_owned(), miner_address.to_owned()],
    )?;
    let value = parse_cli_value(&raw)?;
    value
        .as_array()
        .and_then(|hashes| hashes.first())
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or(LabError::Parse("expected one block hash".to_owned()))
}

/// Return true only when this node's mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    // TODO: inspect getrawmempool.
    let raw = client.call(None, "getrawmempool", &[])?;
    let value = parse_cli_value(&raw)?;
    let mempool = value
        .as_array()
        .ok_or(LabError::Parse("expected an array of TXIDs".to_owned()))?;

    Ok(mempool.is_empty())
}

/// Return a transaction's confirmation count in the selected wallet.
pub fn transaction_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    // TODO: call gettransaction and return confirmations.
    let (confirmations, _) = fetch_transaction(client, wallet_name, txid)?;
    Ok(confirmations)
}

/// Mine, locate the transaction's block, and prove that the block contains the TXID.
pub fn confirm_and_locate_transaction<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
    miner_address: &str,
) -> LabResult<ConfirmationReport> {
    // TODO:
    // 1. Mine one block.
    // 2. Check the mempool.
    // 3. Read gettransaction for blockhash and confirmations.
    // 4. Read getblock and verify that its `tx` array contains txid.
    mine_one_block(client, miner_address)?;
    let is_mempool_empty = mempool_is_empty(client)?;

    let (confirmations, block_hash) = fetch_transaction(client, wallet_name, txid)?;
    let block_hash = block_hash.ok_or(LabError::MissingField("blockhash"))?;

    let raw = client.call(None, "getblock", &[block_hash.clone(), "1".to_owned()])?;
    let value = parse_cli_value(&raw)?;
    let transaction_is_in_block = value
        .get("tx")
        .and_then(Value::as_array)
        .ok_or(LabError::MissingField("tx"))?
        .iter()
        .any(|entry| entry.as_str() == Some(txid));

    Ok(ConfirmationReport {
        txid: txid.to_owned(),
        block_hash,
        confirmations,
        mempool_is_empty: is_mempool_empty,
        transaction_is_in_block,
    })
}
