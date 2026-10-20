//! Lab 07 — confirm a transaction and prove block membership.

use crate::labs::lab03_maturity::mine_blocks;
use crate::labs::lab05_mempool::get_raw_mempool;
use crate::model::ConfirmationReport;
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    let hashes = mine_blocks(client, miner_address, 1)?;
    hashes
        .into_iter()
        .next()
        .ok_or_else(|| LabError::Parse("no block hash returned".to_string()))
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
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let val = parse_cli_value(&raw)?;
    val.get("confirmations")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| LabError::MissingField("confirmations"))
}

/// Mine, locate the transaction's block, and prove that the block contains the TXID.
pub fn confirm_and_locate_transaction<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
    miner_address: &str,
) -> LabResult<ConfirmationReport> {
    let block_hash = mine_one_block(client, miner_address)?;
    let empty_mempool = mempool_is_empty(client)?;
    let confirmations = transaction_confirmations(client, wallet_name, txid)?;

    let raw_block = client.call(None, "getblock", &[block_hash.clone(), "1".to_string()])?;
    let val_block = parse_cli_value(&raw_block)?;

    let tx_arr = val_block
        .get("tx")
        .and_then(|v| v.as_array())
        .ok_or_else(|| LabError::MissingField("tx"))?;

    let transaction_is_in_block = tx_arr.iter().any(|v| v.as_str() == Some(txid));

    Ok(ConfirmationReport {
        txid: txid.to_string(),
        block_hash,
        confirmations,
        mempool_is_empty: empty_mempool,
        transaction_is_in_block,
    })
}
