//! Lab 07 — confirm a transaction and prove block membership.

use crate::model::ConfirmationReport;
use crate::rpc::RpcClient;
use crate::{LabResult, LabError};
use serde::Deserialize;

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    // TODO: call generatetoaddress with a count of one.
    // todo!("Lab 07: mine one block")
    let raw = client.call(
        None,
        "generatetoaddress",
        &["1".to_string(), miner_address.to_string()],
    )?;

    let hashes: Vec<String> =
        serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    hashes
        .into_iter()
        .next()
        .ok_or(LabError::MissingField("generatetoaddress"))
}

#[derive(Deserialize)]
struct RawTransactionWithBlock {
    confirmations: i64,
    blockhash: Option<String>,
}

#[derive(Deserialize)]
struct RawBlock {
    tx: Vec<String>,
}


/// Return true only when this node's mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    // TODO: inspect getrawmempool.
    // todo!("Lab 07: check whether the mempool is empty")
    let raw = client.call(None, "getrawmempool", &[])?;
    let txids: Vec<String> =
        serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    Ok(txids.is_empty())
}

#[derive(Deserialize)]
struct RawTransaction {
    confirmations: i64,
} 

/// Return a transaction's confirmation count in the selected wallet.
pub fn transaction_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    // TODO: call gettransaction and return confirmations.
    // todo!("Lab 07: read transaction confirmations")
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let parsed: RawTransaction =
        serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    Ok(parsed.confirmations)
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
    // todo!("Lab 07: prove confirmation and block membership")
   // 1. Mine one block.
    mine_one_block(client, miner_address)?;

    // 2. Check the mempool.
    let mempool_is_empty_flag = mempool_is_empty(client)?;

    // 3. Read gettransaction for blockhash and confirmations.
    let raw_tx = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let parsed_tx: RawTransactionWithBlock =
        serde_json::from_str(&raw_tx).map_err(|e| LabError::Parse(e.to_string()))?;

    let block_hash = parsed_tx
        .blockhash
        .ok_or(LabError::MissingField("blockhash"))?;
    let confirmations = parsed_tx.confirmations;

    // 4. Read getblock (verbosity 1) and verify that its `tx` array contains txid.
    let raw_block = client.call(
        None,
        "getblock",
        &[block_hash.clone(), "1".to_string()],
    )?;
    let parsed_block: RawBlock =
        serde_json::from_str(&raw_block).map_err(|e| LabError::Parse(e.to_string()))?;

    let transaction_is_in_block = parsed_block.tx.iter().any(|id| id == txid);

    Ok(ConfirmationReport {
        txid: txid.to_string(),
        block_hash,
        confirmations,
        mempool_is_empty: mempool_is_empty_flag,
        transaction_is_in_block,
    })
}
