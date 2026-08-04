//! Lab 07 — confirm a transaction and prove block membership.

use crate::model::ConfirmationReport;
use crate::rpc::RpcClient;
use crate::LabResult;

use crate::rpc::parse_cli_value;
use crate::LabError;
use serde_json::Value;

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    let raw = client.call(
        None,
        "generatetoaddress",
        &["1".to_owned(), miner_address.to_owned()],
    )?;
    let value = parse_cli_value(&raw)?;
    let arr = value
        .as_array()
        .ok_or_else(|| LabError::Parse("Expected array of block hashes".to_owned()))?;
    let block_hash = arr
        .first()
        .and_then(Value::as_str)
        .ok_or_else(|| LabError::Parse("Expected block hash string".to_owned()))?;
    Ok(block_hash.to_owned())
}

/// Return true only when this node's mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    let raw = client.call(None, "getrawmempool", &[])?;
    let value = parse_cli_value(&raw)?;
    let arr = value
        .as_array()
        .ok_or_else(|| LabError::Parse("Expected array of mempool txids".to_owned()))?;
    Ok(arr.is_empty())
}

/// Return a transaction's confirmation count in the selected wallet.
pub fn transaction_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let value = parse_cli_value(&raw)?;
    let confirmations = value
        .get("confirmations")
        .and_then(Value::as_i64)
        .ok_or_else(|| LabError::MissingField("confirmations"))?;
    Ok(confirmations)
}

/// Mine, locate the transaction's block, and prove that the block contains the TXID.
pub fn confirm_and_locate_transaction<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
    miner_address: &str,
) -> LabResult<ConfirmationReport> {
    // 1. Mine one block.
    let block_hash = mine_one_block(client, miner_address)?;

    // 2. Check the mempool.
    let empty = mempool_is_empty(client)?;

    // 3. Read gettransaction for blockhash and confirmations.
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let tx_val = parse_cli_value(&raw)?;
    let confirmations = tx_val
        .get("confirmations")
        .and_then(Value::as_i64)
        .ok_or_else(|| LabError::MissingField("confirmations"))?;
    let got_block_hash = tx_val
        .get("blockhash")
        .and_then(Value::as_str)
        .ok_or_else(|| LabError::MissingField("blockhash"))?;

    // 4. Read getblock and verify that its `tx` array contains txid.
    let block_raw = client.call(
        None,
        "getblock",
        &[got_block_hash.to_owned(), "1".to_owned()],
    )?;
    let block_val = parse_cli_value(&block_raw)?;
    let txs = block_val
        .get("tx")
        .and_then(Value::as_array)
        .ok_or_else(|| LabError::MissingField("tx"))?;
    let transaction_is_in_block = txs.iter().any(|v| v.as_str() == Some(txid));

    Ok(ConfirmationReport {
        txid: txid.to_owned(),
        block_hash: got_block_hash.to_owned(),
        confirmations,
        mempool_is_empty: empty,
        transaction_is_in_block,
    })
}
