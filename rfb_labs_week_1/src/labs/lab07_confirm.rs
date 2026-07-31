//! Lab 07 — confirm a transaction and prove block membership.

use crate::labs::lab03_maturity::mine_blocks;
use crate::labs::lab05_mempool::{get_raw_mempool, get_transaction_status};
use crate::labs::lab_helper::required_array;
use crate::model::ConfirmationReport;
use crate::rpc::{parse_cli_value, required_u64, RpcClient};

use crate::{LabError, LabResult};

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    let blocks_hashes_arr = mine_blocks(client, miner_address, 1)?;
    blocks_hashes_arr
        .into_iter()
        .next()
        .ok_or_else(|| LabError::Parse("Expected exactly one mined block hash".to_string()))
}

/// Return true only when this node's mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    let mempool_txids = get_raw_mempool(client)?;
    Ok(mempool_txids.is_empty())
}

/// Return a transaction's confirmation count in the selected wallet.
pub fn transaction_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let value = parse_cli_value(&raw)?;
    let confirmations_u64 = required_u64(&value, "confirmations")?;
    Ok(confirmations_u64 as i64)
}

/// Mine, locate the transaction's block, and prove that the block contains the TXID.
pub fn confirm_and_locate_transaction<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
    miner_address: &str,
) -> LabResult<ConfirmationReport> {
    // 1. Mine one block.
    let block_hash = mine_one_block(client, miner_address)?;

    // 2. Check the mempool.
    let mempool_is_empty = mempool_is_empty(client)?;

    // 3. Read gettransaction for blockhash and confirmations.
    // let tx_status = get_transaction_status(client, wallet_name, txid)?;
    let confirmations = transaction_confirmations(client, wallet_name, txid)?;

    // 4. Read getblock and verify that its `tx` array contains txid.
    let get_block_raw = client.call(None, "getblock", &[block_hash.clone(), "1".to_string()])?;
    let get_block_value = parse_cli_value(&get_block_raw)?;

    let block_txs = required_array(&get_block_value, "tx")?;

    let transaction_is_in_block = block_txs.iter().any(|t| t.as_str() == Some(txid));

    Ok(ConfirmationReport {
        txid: txid.to_owned(),
        block_hash,
        confirmations,
        mempool_is_empty,
        transaction_is_in_block,
    })
}
