//! Lab 07 — confirm a transaction and prove block membership.

use crate::model::ConfirmationReport;
use crate::rpc::RpcClient;
use crate::LabResult;

use crate::rpc::parse_cli_value;
use crate::LabError;

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    let raw = client.call(None, "generatetoaddress", &["1".to_owned(), miner_address.to_owned()])?;
    let json = parse_cli_value(&raw)?;
    let hashes: Vec<String> = serde_json::from_value(json)?;
    hashes.first().cloned().ok_or_else(|| LabError::Parse("No blocks returned".to_owned()))
}

/// Return true only when this node's mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    let raw = client.call(None, "getrawmempool", &[])?;
    let json = parse_cli_value(&raw)?;
    let txids: Vec<String> = serde_json::from_value(json)?;
    Ok(txids.is_empty())
}

/// Return a transaction's confirmation count in the selected wallet.
pub fn transaction_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let json = parse_cli_value(&raw)?;
    let confirmations = json.get("confirmations")
        .and_then(|v| v.as_i64())
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
    mine_one_block(client, miner_address)?;
    let mempool_empty = mempool_is_empty(client)?;

    let raw_tx = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let json_tx = parse_cli_value(&raw_tx)?;
    let confirmations = json_tx.get("confirmations")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| LabError::MissingField("confirmations"))?;
    let block_hash = json_tx.get("blockhash")
        .and_then(|v| v.as_str())
        .ok_or_else(|| LabError::MissingField("blockhash"))?
        .to_owned();

    let raw_block = client.call(None, "getblock", &[block_hash.clone(), "1".to_owned()])?;
    let json_block = parse_cli_value(&raw_block)?;
    let tx_array = json_block.get("tx")
        .and_then(|v| v.as_array())
        .ok_or_else(|| LabError::MissingField("tx"))?;

    let transaction_is_in_block = tx_array.iter()
        .any(|tx| tx.as_str() == Some(txid));

    Ok(ConfirmationReport {
        txid: txid.to_owned(),
        block_hash,
        confirmations,
        mempool_is_empty: mempool_empty,
        transaction_is_in_block,
    })
}
