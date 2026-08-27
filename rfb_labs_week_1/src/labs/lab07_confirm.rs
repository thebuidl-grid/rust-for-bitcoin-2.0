//! Lab 07 — confirm a transaction and prove block membership.

use crate::model::ConfirmationReport;
use crate::rpc::RpcClient;
use crate::LabResult;

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    let hashes = crate::labs::lab03_maturity::mine_blocks(client, miner_address, 1)?;
    hashes
        .into_iter()
        .next()
        .ok_or_else(|| crate::LabError::Parse("no block hash returned".to_owned()))
}

/// Return true only when this node's mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    let mempool = crate::labs::lab05_mempool::get_raw_mempool(client)?;
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
        .and_then(serde_json::Value::as_i64)
        .ok_or(crate::LabError::MissingField("confirmations"))
}

/// Mine, locate the transaction's block, and prove that the block contains the TXID.
pub fn confirm_and_locate_transaction<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
    miner_address: &str,
) -> LabResult<ConfirmationReport> {
    let block_hash = mine_one_block(client, miner_address)?;
    let empty = mempool_is_empty(client)?;
    let confirmations = transaction_confirmations(client, wallet_name, txid)?;

    let raw = client.call(None, "getblock", &[block_hash.clone(), "1".to_owned()])?;
    let value = crate::rpc::parse_cli_value(&raw)?;
    let tx_array = value
        .get("tx")
        .and_then(serde_json::Value::as_array)
        .ok_or(crate::LabError::MissingField("tx"))?;
    let transaction_is_in_block = tx_array.iter().any(|v| v.as_str() == Some(txid));

    Ok(ConfirmationReport {
        txid: txid.to_owned(),
        block_hash,
        confirmations,
        mempool_is_empty: empty,
        transaction_is_in_block,
    })
}
