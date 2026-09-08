//! Lab 07 — confirm a transaction and prove block membership.

use crate::labs::lab05_mempool::get_raw_mempool;
use crate::model::ConfirmationReport;
use crate::rpc::{parse_cli_value, RpcClient};
use crate::LabResult;

/// Mine exactly one block and return its hash.
pub fn mine_one_block<C: RpcClient>(client: &C, miner_address: &str) -> LabResult<String> {
    // TODO: call generatetoaddress with a count of one.
    // todo!("Lab 07: mine one block")
    let result = client.call(
        None,
        "generatetoaddress",
        &[1.to_string(), miner_address.to_string()],
    );

    match result {
        Ok(res) => {
            let parsed = parse_cli_value(&res)?;
            let vector_of_result = parsed
                .as_array()
                .unwrap()
                .first()
                .map(|elem| elem.as_str().unwrap().to_string())
                .unwrap();
            Ok(vector_of_result)
        }
        Err(err) => Err(err),
    }
}

/// Return true only when this node's mempool contains no transactions.
pub fn mempool_is_empty<C: RpcClient>(client: &C) -> LabResult<bool> {
    // TODO: inspect getrawmempool.
    // todo!("Lab 07: check whether the mempool is empty")
    let res = get_raw_mempool(client).map(|elem| elem.is_empty())?;
    Ok(res)
}

/// Return a transaction's confirmation count in the selected wallet.
pub fn transaction_confirmations<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<i64> {
    // TODO: call gettransaction and return confirmations.
    // todo!("Lab 07: read transaction confirmations")
    let result = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;

    let confirmations = parse_cli_value(&result)?
        .as_object()
        .unwrap()
        .get("confirmations")
        .map(|elem| elem.as_i64().unwrap())
        .unwrap_or_default();

    Ok(confirmations)
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
    let block_hash = mine_one_block(client, &miner_address)?;
    let mempool_is_empty = mempool_is_empty(client)?;
    let result = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;

    let decoded = parse_cli_value(&result)?;

    let confirmations = decoded["confirmations"].as_i64().unwrap();
    let hash = decoded["blockhash"].as_str().unwrap().to_string();
    let block_result = client.call(None, "getblock", &[hash.clone(), "1".to_string()])?;
    let parsed_block = parse_cli_value(&block_result)?;
    let txn_in_block = parsed_block["tx"]
        .as_array()
        .unwrap()
        .iter()
        .any(|elem| elem.as_str().unwrap() == txid);

    Ok(ConfirmationReport {
        txid: txid.to_owned(),
        block_hash: hash,
        confirmations,
        mempool_is_empty,
        transaction_is_in_block: txn_in_block,
    })
}
