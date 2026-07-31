//! Lab 07 — confirm a transaction and prove block membership.

use crate::model::ConfirmationReport;
use crate::rpc::{required_string, RpcClient};
use crate::{LabError, LabResult};

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    let response = client.call(
        None,
        "generatetoaddress",
        &["1".to_string(), miner_address.to_string()],
    )?;
    let block_hashes = serde_json::from_str::<Vec<String>>(&response)?;
    let block_hash = block_hashes
        .into_iter()
        .next()
        .ok_or_else(|| LabError::Parse("mining returned no block hash".to_string()))?;

    Ok(block_hash)
}

/// Return true only when this node's mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    let response = client.call(None, "getrawmempool", &[])?;
    let transaction_ids = serde_json::from_str::<Vec<String>>(&response)?;

    Ok(transaction_ids.is_empty())
}

/// Return a transaction's confirmation count in the selected wallet.
pub fn transaction_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let response = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let transaction: serde_json::Value = serde_json::from_str(&response)?;
    let confirmations = transaction
        .get("confirmations")
        .and_then(serde_json::Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))?;

    Ok(confirmations)
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

    let transaction_response =
        client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let transaction: serde_json::Value = serde_json::from_str(&transaction_response)?;
    let block_hash = required_string(&transaction, "blockhash")?;
    let confirmations = transaction
        .get("confirmations")
        .and_then(serde_json::Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))?;

    let block_response = client.call(None, "getblock", &[block_hash.clone(), "1".to_string()])?;
    let block: serde_json::Value = serde_json::from_str(&block_response)?;
    let block_transaction_values = block
        .get("tx")
        .and_then(serde_json::Value::as_array)
        .ok_or(LabError::MissingField("tx"))?;
    let transaction_is_in_block = block_transaction_values
        .iter()
        .any(|value| value.as_str() == Some(txid));

    Ok(ConfirmationReport {
        txid: txid.to_string(),
        block_hash,
        confirmations,
        mempool_is_empty,
        transaction_is_in_block,
    })
}
