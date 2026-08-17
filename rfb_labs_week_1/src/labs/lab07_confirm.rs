//! Lab 07 — confirm a transaction and prove block membership.

use crate::labs::lab03_maturity::mine_blocks;
use crate::labs::lab05_mempool::get_raw_mempool;
use crate::model::ConfirmationReport;
use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    mine_blocks(client, miner_address, 1)?
        .into_iter()
        .next()
        .ok_or_else(|| LabError::Parse("generatetoaddress returned no block hash".to_owned()))
}

/// Return true only when this node's mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    Ok(get_raw_mempool(client)?.is_empty())
}

/// Return a transaction's confirmation count in the selected wallet.
pub fn transaction_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let transaction = parse_cli_value(&raw)?;

    transaction
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
    let mempool_cleared = mempool_is_empty(client)?;

    // One `gettransaction` yields both the depth and the containing block, so the
    // two pieces of evidence describe the same moment in the chain.
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let transaction = parse_cli_value(&raw)?;
    let confirmations = transaction
        .get("confirmations")
        .and_then(Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))?;
    let block_hash = required_string(&transaction, "blockhash")?;

    // The wallet naming a block is a claim; the block listing the TXID is the proof.
    let raw_block = client.call(None, "getblock", &[block_hash.clone(), "1".to_owned()])?;
    let block = parse_cli_value(&raw_block)?;
    let transaction_is_in_block = block
        .get("tx")
        .and_then(Value::as_array)
        .ok_or(LabError::MissingField("tx"))?
        .iter()
        .any(|entry| entry.as_str() == Some(txid));

    Ok(ConfirmationReport {
        txid: txid.to_owned(),
        block_hash,
        confirmations,
        mempool_is_empty: mempool_cleared,
        transaction_is_in_block,
    })
}
