//! Lab 07 — confirm a transaction and prove block membership.

use crate::model::ConfirmationReport;
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    let response = client.call(
        None,
        "generatetoaddress",
        &["1".to_string(), miner_address.to_owned()],
    )?;
    let value = parse_cli_value(&response)?;

    value
        .as_array()
        .and_then(|blocks| blocks.first())
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or(LabError::Parse(
            "generatetoaddress must return an array with one block hash".to_string(),
        ))
}

/// Return true only when this node's mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    let response = client.call(None, "getrawmempool", &[])?;
    let value = parse_cli_value(&response)?;
    let mempool = value
        .as_array()
        .ok_or(LabError::Parse("getrawmempool must return an array".to_string()))?;
    Ok(mempool.is_empty())
}

/// Return a transaction's confirmation count in the selected wallet.
pub fn transaction_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let response = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let value = parse_cli_value(&response)?;

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

    let tx_response = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let tx_value = parse_cli_value(&tx_response)?;
    let confirmations = tx_value
        .get("confirmations")
        .and_then(serde_json::Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))?;
    let block_hash = tx_value
        .get("blockhash")
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or(LabError::MissingField("blockhash"))?;

    let block_response = client.call(None, "getblock", &[block_hash.clone(), "1".to_string()])?;
    let block_value = parse_cli_value(&block_response)?;
    let txs = block_value
        .get("tx")
        .and_then(serde_json::Value::as_array)
        .ok_or(LabError::MissingField("tx"))?;
    let transaction_is_in_block = txs
        .iter()
        .any(|candidate| candidate.as_str() == Some(txid));

    Ok(ConfirmationReport {
        txid: txid.to_owned(),
        block_hash,
        confirmations,
        mempool_is_empty,
        transaction_is_in_block,
    })
}
