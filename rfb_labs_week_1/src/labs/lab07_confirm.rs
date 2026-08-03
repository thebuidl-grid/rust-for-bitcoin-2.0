//! Lab 07 — confirm a transaction and prove block membership.

use crate::model::ConfirmationReport;
use crate::rpc::{parse_cli_value, RpcClient};
use crate::LabResult;

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
        .ok_or_else(|| crate::LabError::Parse("expected an array of block hashes".to_owned()))?
        .first()
        .and_then(|v| v.as_str())
        .map(|s| s.to_owned())
        .ok_or_else(|| crate::LabError::Parse("expected at least one block hash".to_owned()))
}

/// Return true only when this node's mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    let raw = client.call(None, "getrawmempool", &[])?;
    let value = parse_cli_value(&raw)?;
    Ok(value.as_array().map(|arr| arr.is_empty()).unwrap_or(false))
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
        .and_then(|v| v.as_i64())
        .ok_or_else(|| crate::LabError::MissingField("confirmations"))
}

/// Mine, locate the transaction's block, and prove that the block contains the TXID.
pub fn confirm_and_locate_transaction<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
    miner_address: &str,
) -> LabResult<ConfirmationReport> {
    let block_hash = mine_one_block(client, miner_address)?;
    let mempool_is_empty = mempool_is_empty(client)?;
    let confirmations = transaction_confirmations(client, wallet_name, txid)?;

    let raw = client.call(None, "getblock", &[block_hash.clone(), "1".to_owned()])?;
    let value = parse_cli_value(&raw)?;
    let transaction_is_in_block = value
        .get("tx")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .any(|v| v.as_str().map(|s| s == txid).unwrap_or(false))
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
