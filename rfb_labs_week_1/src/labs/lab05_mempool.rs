//! Lab 05 — broadcast a transaction and observe the mempool.

use serde_json::Value;

use crate::model::{MempoolObservation, WalletBalances, WalletTransactionStatus};
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};

/// Send bitcoin from one wallet and return the TXID.
pub fn send_btc<C: RpcClient>(
    client: &C,
    from_wallet: &str,
    destination: &str,
    amount_btc: f64,
) -> LabResult<String> {
    let raw = client.call(
        Some(from_wallet),
        "sendtoaddress",
        &[destination.to_string(), amount_btc.to_string()],
    )?;
    let value = parse_cli_value(&raw)?;

    match value {
        Value::String(txid) => Ok(txid),
        other => Err(LabError::Parse(format!(
            "expected txid string, got {other}"
        ))),
    }
}

/// Return the node's local mempool as a list of TXIDs.
pub fn get_raw_mempool<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let raw = client.call(None, "getrawmempool", &[])?;
    let value = parse_cli_value(&raw)?;

    match value {
        Value::Array(items) => items
            .into_iter()
            .map(|item| {
                item.as_str()
                    .map(ToOwned::to_owned)
                    .ok_or(LabError::Parse("expected string txid".to_owned()))
            })
            .collect(),
        other => Err(LabError::Parse(format!("expected array, got {other}"))),
    }
}

/// Return the selected wallet's view of one transaction.
pub fn get_transaction_status<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<WalletTransactionStatus> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let value = parse_cli_value(&raw)?;

    Ok(WalletTransactionStatus {
        txid: value
            .get("txid")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .ok_or(LabError::MissingField("txid"))?,
        amount: value
            .get("amount")
            .and_then(Value::as_f64)
            .ok_or(LabError::MissingField("amount"))?,
        fee: value.get("fee").and_then(Value::as_f64),
        confirmations: value
            .get("confirmations")
            .and_then(Value::as_i64)
            .ok_or(LabError::MissingField("confirmations"))?,
        block_hash: value
            .get("blockhash")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
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
    let mempool_contains_tx = mempool.iter().any(|id| id == &txid);
    let sender_status = get_transaction_status(client, sender_wallet, &txid)?;
    let receiver_balances = {
        let raw = client.call(Some(receiver_wallet), "getbalances", &[])?;
        let value = parse_cli_value(&raw)?;
        let mine = value
            .get("mine")
            .and_then(Value::as_object)
            .ok_or(LabError::MissingField("mine"))?;

        WalletBalances {
            trusted: mine
                .get("trusted")
                .and_then(Value::as_f64)
                .ok_or(LabError::MissingField("trusted"))?,
            untrusted_pending: mine
                .get("untrusted_pending")
                .and_then(Value::as_f64)
                .ok_or(LabError::MissingField("untrusted_pending"))?,
            immature: mine
                .get("immature")
                .and_then(Value::as_f64)
                .ok_or(LabError::MissingField("immature"))?,
        }
    };

    Ok(MempoolObservation {
        txid,
        mempool_contains_tx,
        sender_status,
        receiver_balance: receiver_balances,
    })
}
