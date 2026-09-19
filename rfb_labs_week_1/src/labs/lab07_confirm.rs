use crate::model::ConfirmationReport;
use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    let raw = client.call(
        None,
        "generatetoaddress",
        &["1".to_owned(), miner_address.to_owned()],
    )?;
    let value = parse_cli_value(&raw)?;

    value
        .as_array()
        .and_then(|array| array.first())
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| LabError::Parse("expected exactly one block hash".to_owned()))
}

pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    let raw = client.call(None, "getrawmempool", &[])?;
    let value = parse_cli_value(&raw)?;

    Ok(value
        .as_array()
        .map(|array| array.is_empty())
        .unwrap_or(false))
}

pub fn transaction_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let value = parse_cli_value(&raw)?;

    value
        .get("confirmations")
        .and_then(Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))
}

pub fn confirm_and_locate_transaction<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
    miner_address: &str,
) -> LabResult<ConfirmationReport> {
    mine_one_block(client, miner_address)?;
    let mempool_empty = mempool_is_empty(client)?;

    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let value = parse_cli_value(&raw)?;
    let block_hash = required_string(&value, "blockhash")?;
    let confirmations = value
        .get("confirmations")
        .and_then(Value::as_i64)
        .ok_or(LabError::MissingField("confirmations"))?;

    let raw_block = client.call(None, "getblock", &[block_hash.clone(), "1".to_owned()])?;
    let block_value = parse_cli_value(&raw_block)?;
    let transaction_is_in_block = block_value
        .get("tx")
        .and_then(Value::as_array)
        .map(|txs| txs.iter().any(|entry| entry.as_str() == Some(txid)))
        .unwrap_or(false);

    Ok(ConfirmationReport {
        txid: txid.to_owned(),
        block_hash,
        confirmations,
        mempool_is_empty: mempool_empty,
        transaction_is_in_block,
    })
}
