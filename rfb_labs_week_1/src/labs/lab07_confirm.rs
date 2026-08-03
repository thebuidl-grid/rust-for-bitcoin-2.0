//! Lab 07 — confirm a transaction and prove block membership.

use crate::model::ConfirmationReport;
use crate::rpc::{parse_cli_value,required_string, RpcClient};
use crate::{LabError, LabResult};
use crate::labs::lab05_mempool::get_raw_mempool;

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    // TODO: call generatetoaddress with a count of one.
        let raw_info = client.call(
        None,
        "generatetoaddress",
        &["1".to_string(), miner_address.to_string()],
    )?;
    let info = parse_cli_value(&raw_info)?;

    let hashes = info
        .as_array()
        .ok_or(LabError::MissingField("generatetoaddress"))?;

    hashes
        .first()
        .and_then(serde_json::Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or(LabError::MissingField("generatetoaddress[0]"))
}

/// Return true only when this node's mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    // TODO: inspect getrawmempool.
    let mempool = get_raw_mempool(client)?;
    Ok(mempool.is_empty())
}

/// Return a transaction's confirmation count in the selected wallet.
pub fn transaction_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    // TODO: call gettransaction and return confirmations.
    let raw_info = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let info = parse_cli_value(&raw_info)?;

    info.get("confirmations")
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
    // TODO:
    // 1. Mine one block.
    // 2. Check the mempool.
    // 3. Read gettransaction for blockhash and confirmations.
    // 4. Read getblock and verify that its `tx` array contains txid.
    mine_one_block(client, miner_address)?;

    let mempool_is_empty = mempool_is_empty(client)?;

    let raw_tx_info = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let tx_info = parse_cli_value(&raw_tx_info)?;

    let confirmations = tx_info
        .get("confirmations")
        .and_then(serde_json::Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))?;

    let block_hash = required_string(&tx_info, "blockhash")?;

    let raw_block_info = client.call(
        None,
        "getblock",
        &[block_hash.clone(), "1".to_string()],
    )?;
    let block_info = parse_cli_value(&raw_block_info)?;

    let block_txids = block_info
        .get("tx")
        .and_then(serde_json::Value::as_array)
        .ok_or(LabError::MissingField("tx"))?;

    let transaction_is_in_block = block_txids
        .iter()
        .filter_map(serde_json::Value::as_str)
        .any(|entry| entry == txid);

    Ok(ConfirmationReport {
        txid: txid.to_string(),
        block_hash,
        confirmations,
        mempool_is_empty,
        transaction_is_in_block,
    })
}
