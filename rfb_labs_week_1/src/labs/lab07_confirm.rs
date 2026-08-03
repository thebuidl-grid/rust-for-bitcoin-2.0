//! Lab 07 — confirm a transaction and prove block membership.

use crate::model::ConfirmationReport;
use crate::rpc::RpcClient;
use crate::LabError;
use crate::LabResult;

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    // TODO: call generatetoaddress with a count of one.
    //todo!("Lab 07: mine one block")
    let raw = client.call(
        None,
        "generatetoaddress",
        &["1".to_owned(), miner_address.to_owned()],
    )?;
    let value = crate::rpc::parse_cli_value(&raw)?;
    let hashes: Vec<String> = serde_json::from_value(value)?;
    hashes
        .into_iter()
        .next()
        .ok_or_else(|| LabError::Parse("generatetoaddress returned no block hashes".to_owned()))
}

/// Return true only when this node's mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    // TODO: inspect getrawmempool.
    //todo!("Lab 07: check whether the mempool is empty")
    let raw = client.call(None, "getrawmempool", &[])?;
    let value = crate::rpc::parse_cli_value(&raw)?;
    let txids: Vec<String> = serde_json::from_value(value)?;
    Ok(txids.is_empty())
}

/// Return a transaction's confirmation count in the selected wallet.
pub fn transaction_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    // TODO: call gettransaction and return confirmations.
    //todo!("Lab 07: read transaction confirmations")
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let value = crate::rpc::parse_cli_value(&raw)?;
    value
        .get("confirmations")
        .and_then(|v| v.as_i64())
        .ok_or(LabError::MissingField("confirmations"))
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
    //todo!("Lab 07: prove confirmation and block membership")
    mine_one_block(client, miner_address)?;
    let mempool_is_empty = mempool_is_empty(client)?;

    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let value = crate::rpc::parse_cli_value(&raw)?;
    let confirmations = value
        .get("confirmations")
        .and_then(|v| v.as_i64())
        .ok_or(LabError::MissingField("confirmations"))?;
    let block_hash = crate::rpc::required_string(&value, "blockhash")?;

    let block_raw = client.call(None, "getblock", &[block_hash.clone(), "1".to_owned()])?;
    let block_value = crate::rpc::parse_cli_value(&block_raw)?;
    let block_txids: Vec<String> = serde_json::from_value(
        block_value
            .get("tx")
            .cloned()
            .ok_or(LabError::MissingField("tx"))?,
    )?;
    let transaction_is_in_block = block_txids.iter().any(|entry| entry == txid);

    Ok(ConfirmationReport {
        txid: txid.to_owned(),
        block_hash,
        confirmations,
        mempool_is_empty,
        transaction_is_in_block,
    })
}
