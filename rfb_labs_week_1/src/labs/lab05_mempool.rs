//! Lab 05 — broadcast a transaction and observe the mempool.

use crate::model::{MempoolObservation, WalletBalances, WalletTransactionStatus};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
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
        &[destination.to_string(), amount_btc.to_string()],
    )?;

    let val = parse_cli_value(&raw)?;
    if let Some(txid) = val.as_str() {
        Ok(txid.to_string())
    } else {
        Ok(raw)
    }
}

/// Return the node's local mempool as a list of TXIDs.
pub fn get_raw_mempool<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    let raw = client.call(None, "getrawmempool", &[])?;
    let val = parse_cli_value(&raw)?;

    serde_json::from_value::<Vec<String>>(val).map_err(Into::into)
}

/// Return the selected wallet's view of one transaction.
pub fn get_transaction_status<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<WalletTransactionStatus> {
    let raw = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()])?;
    let val = parse_cli_value(&raw)?;

    let txid = required_string(&val, "txid")?;

    // confirmations can be 0 or positive/negative, map to i64
    let confirmations = val
        .get("confirmations")
        .and_then(Value::as_i64)
        .unwrap_or(0);

    let amount = required_f64(&val, "amount")?;
    let fee = val.get("fee").and_then(Value::as_f64);
    let block_hash = val
        .get("blockhash")
        .and_then(|v| v.as_str())
        .map(ToOwned::to_owned);

    Ok(WalletTransactionStatus {
        txid,
        confirmations,
        amount,
        fee,
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
    // 1. Send transaction from sender
    let txid = send_btc(client, sender_wallet, receiver_address, amount_btc)?;

    // 2. Inspect mempool
    let raw_mempool = get_raw_mempool(client)?;
    let mempool_contains_tx = raw_mempool.contains(&txid);

    // 3. Get sender transaction status
    let sender_status = get_transaction_status(client, sender_wallet, &txid)?;

    // 4. Get receiver wallet balances (should show as untrusted_pending)
    let raw_balances = client.call(Some(receiver_wallet), "getbalances", &[])?;
    let balances_val = parse_cli_value(&raw_balances)?;

    let mine = balances_val
        .get("mine")
        .ok_or(crate::LabError::MissingField("mine"))?;

    let trusted = required_f64(mine, "trusted")?;
    let untrusted_pending = required_f64(mine, "untrusted_pending")?;
    let immature = required_f64(mine, "immature")?;

    let receiver_balance = WalletBalances {
        trusted,
        untrusted_pending,
        immature,
    };

    Ok(MempoolObservation {
        txid,
        mempool_contains_tx,
        sender_status,
        receiver_balance,
    })
}
