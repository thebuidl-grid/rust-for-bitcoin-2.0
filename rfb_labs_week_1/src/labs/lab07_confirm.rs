//! Lab 07 — confirm a transaction and prove block membership.

use crate::labs::lab05_mempool::{get_raw_mempool, get_transaction_status};
use crate::model::ConfirmationReport;
use crate::rpc::RpcClient;
use crate::{LabError, LabResult};

/// Mine exactly one block and return its hash.

pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    let raw_response = client.call(
        None,
        "generatetoaddress",
        &["1".to_string(), miner_address.to_string()], // 👈 "1" MUST be the first parameter!
    )?;

    let block_hashes: Vec<String> = serde_json::from_str(&raw_response)?;
    block_hashes
        .into_iter()
        .next()
        .ok_or_else(|| LabError::MissingField("mined block hash array was empty"))
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
    let status = get_transaction_status(client, wallet_name, txid)?;
    Ok(status.confirmations)
}

/// Mine, locate the transaction's block, and prove that the block contains the TXID.
pub fn confirm_and_locate_transaction<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
    miner_address: &str,
) -> LabResult<ConfirmationReport> {
    let mined_block_hash = mine_one_block(client, miner_address)?;
    let empty_mempool = mempool_is_empty(client)?;
    let status = get_transaction_status(client, wallet_name, txid)?;

    let block_hash = status
        .block_hash
        .unwrap_or(mined_block_hash);

    // Pass block_hash AND "1" for verbosity to match the test mock expectation
    let raw_block = client.call(None, "getblock", &[block_hash.clone(), "1".to_string()])?;
    let block_val: serde_json::Value = serde_json::from_str(&raw_block)?;

    let transaction_is_in_block = block_val["tx"]
        .as_array()
        .map(|txs| txs.iter().any(|t| t.as_str() == Some(txid)))
        .unwrap_or(false);

    Ok(ConfirmationReport {
        txid: txid.to_string(),
        block_hash,
        confirmations: status.confirmations,
        mempool_is_empty: empty_mempool,
        transaction_is_in_block,
    })
}