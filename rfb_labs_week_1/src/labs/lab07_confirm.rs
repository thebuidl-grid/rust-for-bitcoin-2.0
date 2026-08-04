//! Lab 07 — confirm a transaction and prove block membership.

use crate::labs::lab05_mempool::get_raw_mempool;
use crate::model::ConfirmationReport;
use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::{LabError, LabResult};

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
     let call = client.call(
        None,
        "generatetoaddress",
        &["1".to_string(), miner_address.to_string()],
    )?;
    let response = parse_cli_value(&call)?;

    let blocks = response
        .as_array()
        .ok_or_else(|| LabError::Parse("expected array".to_string()))?;
    let block_hash = blocks[0]
        .as_str()
        .ok_or_else(|| LabError::Parse("expected string".to_string()))?;
    Ok(block_hash.to_string())
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
      let call = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let response = parse_cli_value(&call)?;

    response
        .get("confirmations")
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
    let mempool_empty = mempool_is_empty(client)?;

    let call = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let response = parse_cli_value(&call)?;

    let confirmations = response
        .get("confirmations")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| LabError::MissingField("confirmations"))?;

    let tx_block_hash = required_string(&response, "blockhash")?;

    let call = client.call(None, "getblock", &[tx_block_hash.clone(), "1".to_string()])?;
    let block_response = parse_cli_value(&call)?;

    let tx_array = block_response
        .get("tx")
        .and_then(|v| v.as_array())
        .ok_or_else(|| LabError::MissingField("tx"))?;

    let transaction_is_in_block = tx_array
        .iter()
        .any(|v| v.as_str().map(|s| s == txid).unwrap_or(false));
    Ok(ConfirmationReport {
        txid: txid.to_string(),
        block_hash: tx_block_hash,
        confirmations,
        mempool_is_empty: mempool_empty,
        transaction_is_in_block,
    })
}
