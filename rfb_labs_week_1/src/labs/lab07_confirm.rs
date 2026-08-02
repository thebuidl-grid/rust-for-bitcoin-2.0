//! Lab 07 — confirm a transaction and prove block membership.

use crate::labs::lab03_maturity::mine_blocks;
use crate::labs::lab05_mempool::get_raw_mempool;
use crate::model::ConfirmationReport;
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    // 1. Mine 1 block using the mine_blocks helper from lab_03
    let hashes = mine_blocks(client, miner_address, 1)?;

    // 2. Return the single block hash generated
    hashes
        .into_iter()
        .next()
        .ok_or_else(|| LabError::Parse("generatetoaddress returned an empty array".to_owned()))
}

/// Return true only when this node's mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    // 1. Fetch raw mempool txids using get_raw_mempool helper from lab_05
    let mempool = get_raw_mempool(client)?;

    // 2. Return true if mempool is empty
    Ok(mempool.is_empty())
}

/// Return a transaction's confirmation count in the selected wallet.
pub fn transaction_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    // 1. Retrieve the transaction via wallet context
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.into()])?;
    let value = parse_cli_value(&raw)?;

    // 2. Return confirmation count
    value["confirmations"]
        .as_i64()
        .ok_or_else(|| LabError::Parse("gettransaction missing integer 'confirmations'".to_owned()))
}

/// Mine, locate the transaction's block, and prove that the block contains the TXID.
pub fn confirm_and_locate_transaction<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
    miner_address: &str,
) -> LabResult<ConfirmationReport> {
    // 1. Mine one block to package unconfirmed transactions from the mempool
    let mined_block_hash = mine_one_block(client, miner_address)?;

    // 2. Verify that the mempool was cleared
    let empty_mempool = mempool_is_empty(client)?;

    // 3. Inspect transaction via wallet to get confirmation count and inclusion blockhash
    let raw_tx = client.call(Some(wallet_name), "gettransaction", &[txid.into()])?;
    let val_tx = parse_cli_value(&raw_tx)?;

    let confirmations = val_tx["confirmations"]
        .as_i64()
        .ok_or_else(|| LabError::Parse("gettransaction missing integer 'confirmations'".to_owned()))?;

    let block_hash = val_tx["blockhash"].as_str().map(ToOwned::to_owned).ok_or_else(|| {
        LabError::Parse("transaction status missing blockhash after mining".to_owned())
    })?;

    // Verify the transaction was indeed included in the block we just mined
    if block_hash != mined_block_hash {
        return Err(LabError::Parse(format!(
            "blockhash mismatch: mined {mined_block_hash}, but transaction recorded {block_hash}"
        )));
    }

    // 4. Query `getblock` (verbosity 1 / default JSON format) to verify `tx` array inclusion
    let raw_block = client.call(None, "getblock", &[block_hash.clone(), 1.to_string()])?;
    let val_block = parse_cli_value(&raw_block)?;

    let tx_array = val_block["tx"]
        .as_array()
        .ok_or_else(|| LabError::Parse("getblock response missing 'tx' array".to_owned()))?;

    let in_block = tx_array.iter().any(|val| val.as_str() == Some(txid));

    Ok(ConfirmationReport {
        txid: txid.to_owned(),
        block_hash,
        confirmations,
        mempool_is_empty: empty_mempool,
        transaction_is_in_block: in_block,
    })
}