//! Lab 05 — broadcast a transaction and observe the mempool.

use crate::labs::lab03_maturity::get_balances;
use crate::model::{MempoolObservation, WalletBalances, WalletTransactionStatus};
use crate::rpc::{parse_cli_value, required_f64, required_string, RpcClient};
use crate::{LabError, LabResult};

/// Send bitcoin from one wallet and return the TXID.
pub fn send_btc<C: RpcClient>(
    client: &C,
    from_wallet: &str,
    destination: &str,
    amount_btc: f64,
) -> LabResult<String> {
    // 1. Call `sendtoaddress` inside the sender's wallet context
    let raw = client.call(
        Some(from_wallet),
        "sendtoaddress",
        &[destination.into(), amount_btc.to_string()],
    )?;
    let value = parse_cli_value(&raw)?;

    // 2. Decode the TXID string returned by Bitcoin Core
    value
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| LabError::Parse("sendtoaddress did not return a txid string".to_owned()))
}

/// Return the node's local mempool as a list of TXIDs.
pub fn get_raw_mempool<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    // 1. Call `getrawmempool` with no arguments (global node level, no wallet context)
    let raw = client.call(None, "getrawmempool", &[])?;
    let value = parse_cli_value(&raw)?;

    // 2. Decode the array of TXIDs
    let txids = value
        .as_array()
        .ok_or_else(|| LabError::Parse("getrawmempool did not return an array".to_owned()))?
        .iter()
        .filter_map(|val| val.as_str().map(ToOwned::to_owned))
        .collect();

    Ok(txids)
}

/// Return the selected wallet's view of one transaction.
pub fn get_transaction_status<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<WalletTransactionStatus> {
    // 1. Call `gettransaction` in the wallet context
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.into()])?;
    let value = parse_cli_value(&raw)?;

    // 2. Decode transaction details
    let returned_txid = required_string(&value, "txid")?;
    let amount = required_f64(&value, "amount")?;

    // Fee is optional/absent for some transaction views, but present on sent payments (negative float)
    let fee = value.get("fee").and_then(|v| v.as_f64());

    let confirmations = value["confirmations"]
        .as_i64()
        .ok_or_else(|| LabError::Parse("gettransaction missing integer 'confirmations'".to_owned()))?;

    // blockhash is only present if confirmations > 0
    let block_hash = value["blockhash"].as_str().map(ToOwned::to_owned);

    Ok(WalletTransactionStatus {
        txid: returned_txid,
        amount,
        fee,
        confirmations,
        block_hash,
    })
}

/// Send a payment without mining and capture its mempool and receiver-wallet state.
pub fn observe_unconfirmed_payment<C: RpcClient>(
    client: &C,
    sender_wallet: &str,
    receiver_wallet: &str,
    receiver_address: &str,
    amount_btc: f64,
) -> LabResult<MempoolObservation> {
    // 1. Broadcast payment from sender to receiver
    let txid = send_btc(client, sender_wallet, receiver_address, amount_btc)?;

    // 2. Query mempool to verify the txid is present in the waiting room
    let mempool_txids = get_raw_mempool(client)?;
    let in_mempool = mempool_txids.contains(&txid);

    // 3. Inspect sender transaction status (confirmations should be 0)
    let sender_tx_status = get_transaction_status(client, sender_wallet, &txid)?;

    // 4. Read receiver wallet balances (should reflect under `untrusted_pending`)
    let receiver_balances = get_balances(client, receiver_wallet)?;

    Ok(MempoolObservation {
        txid,
        mempool_contains_tx: in_mempool,
        sender_status: sender_tx_status,
        receiver_balance: receiver_balances,
    })
}