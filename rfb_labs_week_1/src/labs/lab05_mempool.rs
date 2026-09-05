//! Lab 05 — broadcast a transaction and observe the mempool.

use crate::model::{MempoolObservation, WalletBalances, WalletTransactionStatus};
use crate::rpc::{parse_cli_value, required_string, RpcClient};
use crate::{LabError, LabResult};

/// Send bitcoin from one wallet and return the TXID.
pub fn send_btc<C: RpcClient>(
    client: &C,
    from_wallet: &str,
    destination: &str,
    amount_btc: f64,
) -> LabResult<String> {
    let call = client.call(
        Some(from_wallet),
        "sendtoaddress",
        &[destination.to_string(), amount_btc.to_string()],
    )?;
    let val = parse_cli_value(&call)?;

    val.as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| LabError::Parse("sendtoaddress response is not a string".to_string()))
}

/// Return the node's local mempool as a list of TXIDs.
pub fn get_raw_mempool<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let call = client.call(None, "getrawmempool", &[])?;
    let val = parse_cli_value(&call)?;

    serde_json::from_value::<Vec<String>>(val).map_err(Into::into)
}

/// Return the selected wallet's view of one transaction.
pub fn get_transaction_status<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<WalletTransactionStatus> {
    let call = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let val = parse_cli_value(&call)?;

    let txid = required_string(&val, "txid")?;

    let amount = val
        .get("amount")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| LabError::MissingField("amount"))?;

    let fee = val.get("fee").and_then(|v| v.as_f64());

    // confirmations can be 0 or positive/negative, map to i64
    let confirmations = val
        .get("confirmations")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| LabError::MissingField("confirmations"))?;

    let block_hash = val
        .get("blockhash")
        .and_then(|v| v.as_str())
        .map(String::from);

    Ok(WalletTransactionStatus {
        txid,
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
    let txid = send_btc(client, sender_wallet, receiver_address, amount_btc)?;

    let mempool = get_raw_mempool(client)?;
    let mempool_contains_tx = mempool.contains(&txid);

    let sender_status = get_transaction_status(client, sender_wallet, &txid)?;

    let call = client.call(Some(receiver_wallet), "getbalances", &[])?;
    let val = parse_cli_value(&call)?;
    let mine = val
        .get("mine")
        .ok_or_else(|| LabError::MissingField("mine"))?;
    let receiver_balance = serde_json::from_value::<WalletBalances>(mine.clone())?;

    Ok(MempoolObservation {
        txid,
        sender_status,
        mempool_contains_tx,
        receiver_balance,
    })
}
