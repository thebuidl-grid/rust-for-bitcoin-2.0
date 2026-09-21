//! Lab 07 — confirm a transaction and prove block membership.

use crate::model::ConfirmationReport;
use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
        let raw = client.call(
        None,
        "generatetoaddress",
        &["1".to_string(), miner_address.to_string()],
    )?;
    let val = parse_cli_value(&raw)?;

    let hashes = val.as_array().ok_or_else(|| {
        LabError::Parse("expected array of block hashes from generatetoaddress".to_string())
    })?;

    let first_hash = hashes
        .first()
        .and_then(|h| h.as_str())
        .ok_or(LabError::MissingField("first block hash in array"))?;

    Ok(first_hash.to_string())
}

/// Return true only when this node's mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    let raw = client.call(None, "getrawmempool", &[])?;
    let val = parse_cli_value(&raw)?;

    let mempool_txs = serde_json::from_value::<Vec<String>>(val).map_err(LabError::from)?;
    Ok(mempool_txs.is_empty())
}

/// Return a transaction's confirmation count in the selected wallet.
pub fn transaction_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    // TODO: call gettransaction and return confirmations.
     let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let val = parse_cli_value(&raw)?;

    let confirmations = val
        .get("confirmations")
        .and_then(Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))?;

    Ok(confirmations)
}

/// Mine, locate the transaction's block, and prove that the block contains the TXID.
pub fn confirm_and_locate_transaction<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
    miner_address: &str,
) -> LabResult<ConfirmationReport> {
  let mined_block_hash = mine_one_block(client, miner_address)?;

    // 2. Check if the mempool is empty
    let mempool_empty = mempool_is_empty(client)?;

    // 3. Get wallet transaction status
    let raw_tx = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let tx_val = parse_cli_value(&raw_tx)?;

    let block_hash = required_string(&tx_val, "blockhash")?;
    let confirmations = tx_val
        .get("confirmations")
        .and_then(Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))?;

    if block_hash != mined_block_hash {
        return Err(LabError::Parse(format!(
            "transaction block hash ({block_hash}) does not match mined block hash ({mined_block_hash})"
        )));
    }

    // 4. Fetch the block details (verbosity 1) to inspect the tx array
    let raw_block = client.call(None, "getblock", &[block_hash.clone(), "1".to_string()])?;
    let block_val = parse_cli_value(&raw_block)?;

    let tx_array = block_val
        .get("tx")
        .and_then(|v| v.as_array())
        .ok_or(LabError::MissingField("tx"))?;

    let transaction_is_in_block = tx_array.iter().any(|tx| tx.as_str() == Some(txid));

    Ok(ConfirmationReport {
        txid: txid.to_string(),
        block_hash,
        confirmations,
        mempool_is_empty: mempool_empty,
        transaction_is_in_block,
    })
}
