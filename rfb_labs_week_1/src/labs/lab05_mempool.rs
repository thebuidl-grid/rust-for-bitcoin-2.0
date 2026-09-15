//! Lab 05 — broadcast a transaction and observe the mempool.

use crate::labs::lab03_maturity::{attempt_payment, get_balances};
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
    attempt_payment(client, from_wallet, destination, amount_btc)
}

/// Return the node's local mempool as a list of TXIDs.
pub fn get_raw_mempool<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let raw = client.call(None, "getrawmempool", &[])?;
    Ok(serde_json::from_str(&raw)?)
}

/// Return the selected wallet's view of one transaction.
pub fn get_transaction_status<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<WalletTransactionStatus> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_owned()])?;
    let transaction = parse_cli_value(&raw)?;

    Ok(WalletTransactionStatus {
        txid: required_string(&transaction, "txid")?,
        // Signed, because Bitcoin Core reports -1 for a conflicted transaction.
        confirmations: transaction
            .get("confirmations")
            .and_then(Value::as_i64)
            .ok_or(LabError::MissingField("confirmations"))?,
        amount: required_f64(&transaction, "amount")?,
        // Absent for incoming payments: only the spender pays the fee.
        fee: transaction.get("fee").and_then(Value::as_f64),
        // Absent while the transaction is still unconfirmed.
        block_hash: transaction
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
    let sender_status = get_transaction_status(client, sender_wallet, &txid)?;
    let receiver_balance = get_balances(client, receiver_wallet)?;

    Ok(MempoolObservation {
        mempool_contains_tx: mempool.iter().any(|entry| entry == &txid),
        txid,
        sender_status,
        receiver_balance,
    })
}
