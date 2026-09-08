//! Lab 07 — confirm a transaction and prove block membership.

use crate::model::ConfirmationReport;
use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::{LabError, LabResult};

use super::lab03_maturity::mine_blocks;
use super::lab05_mempool::get_raw_mempool;

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
    let response = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let parsed_response = parse_cli_value(&response)?;

    parsed_response
        .get("confirmations")
        .and_then(|value| value.as_i64())
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
    let mempool_is_empty = mempool_is_empty(client)?;

    let transaction_response =
        client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let transaction = parse_cli_value(&transaction_response)?;
    let block_hash = required_string(&transaction, "blockhash")?;
    let confirmations = transaction
        .get("confirmations")
        .and_then(|value| value.as_i64())
        .ok_or(LabError::MissingField("confirmations"))?;

    let block_response = client.call(None, "getblock", &[block_hash.clone(), "1".to_owned()])?;
    let block = parse_cli_value(&block_response)?;
    let transaction_is_in_block = block
        .get("tx")
        .and_then(|value| value.as_array())
        .ok_or(LabError::MissingField("tx"))?
        .iter()
        .any(|value| value.as_str() == Some(txid));

    Ok(ConfirmationReport {
        txid: txid.to_owned(),
        block_hash,
        confirmations,
        mempool_is_empty,
        transaction_is_in_block,
    })
}
