//! Lab 07 — confirm a transaction and prove block membership.
//Author: Yankho Ngolleka - Github: codaMW

use crate::labs::lab05_mempool::get_raw_mempool;
use crate::model::ConfirmationReport;
use crate::rpc::{required_string, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    let raw = client.call(
        None,
        "generatetoaddress",
        &["1".to_owned(), miner_address.to_owned()],
    )?;
    let value = crate::rpc::parse_cli_value(&raw)?;
    let entries = value
        .as_array()
        .ok_or(LabError::MissingField("generatetoaddress"))?;

    entries
        .first()
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or(LabError::MissingField("block hash"))
}

/// Return true only when this node's mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    let mempool = get_raw_mempool(client)?;
    Ok(mempool.is_empty())
}

/// Return a transaction's confirmation count in the selected wallet.
pub fn transaction_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let value = crate::rpc::parse_cli_value(&raw)?;
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
    mine_one_block(client, miner_address)?;
    let mempool_empty = mempool_is_empty(client)?;

    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let value = crate::rpc::parse_cli_value(&raw)?;
    let block_hash = required_string(&value, "blockhash")?;
    let confirmations = value
        .get("confirmations")
        .and_then(Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))?;

    let block_raw = client.call(None, "getblock", &[block_hash.clone(), "1".to_owned()])?;
    let block_value = crate::rpc::parse_cli_value(&block_raw)?;
    let transactions = block_value
        .get("tx")
        .and_then(Value::as_array)
        .ok_or(LabError::MissingField("tx"))?;
    let transaction_is_in_block = transactions
        .iter()
        .any(|entry| entry.as_str() == Some(txid));

    Ok(ConfirmationReport {
        txid: txid.to_owned(),
        block_hash,
        confirmations,
        mempool_is_empty: mempool_empty,
        transaction_is_in_block,
    })
}
