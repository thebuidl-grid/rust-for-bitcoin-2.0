//! Lab 05 — broadcast a transaction and observe the mempool.

use crate::labs::lab03_maturity::get_balances;
use crate::model::{MempoolObservation, WalletTransactionStatus};
use crate::rpc::{parse_cli_value, required_f64, required_string, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

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
        &[destination.to_owned(), format_amount(amount_btc)],
    )?;
    parse_cli_value(&raw)?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| LabError::Parse(format!("sendtoaddress returned `{raw}`")))
}

/// Return the node's local mempool as a list of TXIDs.
pub fn get_raw_mempool<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let raw = client.call(None, "getrawmempool", &[])?;
    let value = parse_cli_value(&raw)?;

    value
        .as_array()
        .ok_or_else(|| LabError::Parse("getrawmempool did not return an array".to_owned()))?
        .iter()
        .map(|entry| {
            entry.as_str().map(ToOwned::to_owned).ok_or_else(|| {
                LabError::Parse("getrawmempool returned a non-string TXID".to_owned())
            })
        })
        .collect()
}

/// Return the selected wallet's view of one transaction.
pub fn get_transaction_status<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<WalletTransactionStatus> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let value = parse_cli_value(&raw)?;

    Ok(WalletTransactionStatus {
        txid: required_string(&value, "txid")?,
        // Confirmations are signed: a conflicted transaction reports a negative depth.
        confirmations: value
            .get("confirmations")
            .and_then(Value::as_i64)
            .ok_or(LabError::MissingField("confirmations"))?,
        amount: required_f64(&value, "amount")?,
        // A wallet only knows the fee of a transaction it sent itself.
        fee: value.get("fee").and_then(Value::as_f64),
        // Present only once the transaction has been mined into a block.
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

    // Broadcast is not confirmation: the transaction is only queued in the mempool,
    // the sender still reports zero confirmations, and the receiver sees the incoming
    // amount as untrusted-pending rather than as a trusted balance.
    let mempool_contains_tx = get_raw_mempool(client)?.iter().any(|entry| entry == &txid);
    let sender_status = get_transaction_status(client, sender_wallet, &txid)?;
    let receiver_balance = get_balances(client, receiver_wallet)?;

    Ok(MempoolObservation {
        txid,
        mempool_contains_tx,
        sender_status,
        receiver_balance,
    })
}

/// Render a BTC amount the way `bitcoin-cli` expects it.
fn format_amount(amount_btc: f64) -> String {
    let rendered = format!("{amount_btc:.8}");
    rendered
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}
