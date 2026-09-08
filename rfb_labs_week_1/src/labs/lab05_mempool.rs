//! Lab 05 — broadcast a transaction and observe the mempool.

use crate::labs::lab03_maturity::get_balances;
use crate::model::{MempoolObservation, WalletBalances, WalletTransactionStatus};
use crate::rpc::{parse_cli_value, RpcClient};
use crate::LabResult;

/// Send bitcoin from one wallet and return the TXID.
pub fn send_btc<C: RpcClient>(
    client: &C,
    from_wallet: &str,
    destination: &str,
    amount_btc: f64,
) -> LabResult<String> {
    // TODO: call sendtoaddress in the sender's wallet context.
    // todo!("Lab 05: send bitcoin")
    let result = client.call(
        Some(from_wallet),
        "sendtoaddress",
        &[destination.to_string(), amount_btc.to_string()],
    );

    match result {
        Ok(res) => Ok(res),
        Err(err) => Err(err),
    }
}

/// Return the node's local mempool as a list of TXIDs.
pub fn get_raw_mempool<C: RpcClient>(client: &C) -> LabResult<Vec<String>> {
    // TODO: call getrawmempool and decode its array.
    // todo!("Lab 05: inspect the local mempool")
    let result = client.call(None, "getrawmempool", &[]);

    match result {
        Ok(res) => {
            let parsed = parse_cli_value(&res)?;
            let vector_of_result = parsed
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_str().unwrap().to_string())
                .collect::<Vec<String>>();

            Ok(vector_of_result)
        }
        Err(err) => Err(err),
    }
}

/// Return the selected wallet's view of one transaction.
pub fn get_transaction_status<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    txid: &str,
) -> LabResult<WalletTransactionStatus> {
    // TODO: call gettransaction and decode txid, amount, fee, confirmations, and blockhash.
    // todo!("Lab 05: inspect wallet transaction status")
    let result = client.call(Some(wallet_name), "gettransaction", &[txid.to_string()]);

    match result {
        Ok(res) => {
            let parsed = parse_cli_value(&res)?;
            let parsed_obj = parsed.as_object().unwrap();
            let txid = parsed_obj["txid"].as_str().unwrap().to_string();
            let confirmations = parsed_obj["confirmations"].as_i64().unwrap();
            let amount = parsed_obj["amount"].as_f64().unwrap();
            let fee = parsed_obj["fee"].as_f64().unwrap();

            Ok(WalletTransactionStatus {
                txid,
                confirmations,
                amount,
                fee: Some(fee),
                block_hash: None,
            })
        }
        Err(err) => Err(err),
    }
}

/// Send a payment without mining and capture its mempool and receiver-wallet state.
pub fn observe_unconfirmed_payment<C: RpcClient>(
    client: &C,
    sender_wallet: &str,
    receiver_wallet: &str,
    receiver_address: &str,
    amount_btc: f64,
) -> LabResult<MempoolObservation> {
    // TODO: send, inspect getrawmempool, inspect sender status, and read receiver balances.
    // todo!("Lab 05: prove a payment is broadcast but unconfirmed")

    let txid = send_btc(client, sender_wallet, receiver_address, 1.0)?;
    let get_raw_mempool = get_raw_mempool(client)?;
    let sender_status = match get_transaction_status(client, sender_wallet, txid.as_str()) {
        Ok(status) => status,
        Err(err) => return Err(err),
    };

    let receiver_balance = get_balances(client, receiver_wallet)?;

    Ok(MempoolObservation {
        txid: txid.clone(),
        mempool_contains_tx: get_raw_mempool.contains(&txid),
        sender_status,
        receiver_balance,
    })
}
