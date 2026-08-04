//! Lab 05 — broadcast a transaction and observe the mempool.

use crate::model::{MempoolObservation, WalletBalances, WalletTransactionStatus};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::labs::lab03_maturity::get_balances;
use crate::{LabError, LabResult};
use serde_json::Value;

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
    let cli_response = parse_cli_value(&call)?;

    match cli_response {
        Value::String(s) => Ok(s),
        _ => Err(LabError::Parse("expected txid string".to_string())),
    }
}

/// Return the node's local mempool as a list of TXIDs.
pub fn get_raw_mempool<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let call = client.call(None, "getrawmempool", &[])?;
    let response = parse_cli_value(&call)?;

    let txids_array = response
        .as_array()
        .ok_or_else(|| LabError::Parse("expected array".to_string()))?;

    txids_array
        .iter()
        .map(|v| {
            v.as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| LabError::Parse("expected string in array".to_string()))
        })
        .collect()
}

/// Return the selected wallet's view of one transaction.
pub fn get_transaction_status<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<WalletTransactionStatus> {
     let call = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let response = parse_cli_value(&call)?;

    let blockhash = if required_string(&response, "blockhash").is_ok() {
        Some(required_string(&response, "blockhash")?)
    } else {
        None
    };

    Ok(WalletTransactionStatus {
        txid: required_string(&response, "txid")?,
        confirmations: required_u64(&response, "confirmations")? as i64,
        amount: required_f64(&response, "amount")?,
        fee: Some(required_f64(&response, "fee")?),
        block_hash: blockhash,
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
     // Send the payment
    let txid = send_btc(client, sender_wallet, receiver_address, amount_btc)?;

    // Check if transaction is in mempool
    let mempool = get_raw_mempool(client)?;
    let mempool_contains_tx = mempool.contains(&txid);

    // Get sender's view of the transaction
    let sender_status = get_transaction_status(client, sender_wallet, &txid)?;

    // Get receiver's balance (should show untrusted_pending)
    let receiver_balance = get_balances(client, receiver_wallet)?;

    Ok(MempoolObservation {
        txid,
        mempool_contains_tx,
        sender_status,
        receiver_balance,
    })
}
