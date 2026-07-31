//! Lab 07 — confirm a transaction and prove block membership.

use crate::model::ConfirmationReport;
use crate::rpc::{parse_cli_value, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    let response = client.call(
        None,
        "generatetoaddress",
        &["1".to_string(), miner_address.to_string()],
    )?;

    let value = parse_cli_value(&response)?;

    Ok(value[0]
        .as_str()
        .ok_or(LabError::Parse("expected block hash".into()))?
        .to_string())
}

/// Return true only when this node's mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    let response = client.call(None, "getrawmempool", &[])?;
    let value = parse_cli_value(&response)?;

    Ok(value.as_array().map(|a| a.is_empty()).unwrap_or(false))
}

/// Return a transaction's confirmation count in the selected wallet.
pub fn transaction_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let response = client.call(
        Some(wallet_name),
        "gettransaction",
        &[txid.to_string()],
    )?;

    let value = parse_cli_value(&response)?;

    Ok(required_u64(&value, "confirmations")? as i64)
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

    let response = client.call(
        Some(wallet_name),
        "gettransaction",
        &[txid.to_string()],
    )?;
    let value = parse_cli_value(&response)?;

    let block_hash = required_string(&value, "blockhash")?;
    let confirmations = required_u64(&value, "confirmations")? as i64;

    let response = client.call(
        None,
        "getblock",
        &[block_hash.clone(), "1".to_string()],
    )?;
    let block = parse_cli_value(&response)?;

    let transaction_is_in_block = block["tx"]
        .as_array()
        .map(|txs| txs.iter().any(|tx| tx.as_str() == Some(txid)))
        .unwrap_or(false);

    Ok(ConfirmationReport {
        txid: txid.to_string(),
        block_hash,
        confirmations,
        mempool_is_empty: mempool_empty,
        transaction_is_in_block,
    })
}