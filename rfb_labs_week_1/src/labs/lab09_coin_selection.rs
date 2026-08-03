use crate::labs::lab04_utxos::list_unspent;
use crate::labs::lab06_decode::{
    calculate_fee, decode_verbose_transaction, identify_payment_and_change,
};
use crate::model::{MultiUtxoAudit, Utxo};
use crate::rpc::RpcClient;
use crate::LabResult;

pub fn create_three_funding_transactions<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    alice_address: &str,
) -> LabResult<Vec<String>> {
    (0..3)
        .map(|_| {
            client.call(
                Some(miner_wallet),
                "sendtoaddress",
                &[alice_address.to_owned(), 0.4_f64.to_string()],
            )
        })
        .collect()
}

pub fn confirmed_utxos_for_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<Vec<Utxo>> {
    let utxos = list_unspent(client, wallet_name)?;

    Ok(utxos
        .into_iter()
        .filter(|utxo| utxo.confirmations > 0 && utxo.address.as_deref() == Some(address))
        .collect())
}

pub fn send_combined_payment<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
) -> LabResult<String> {
    client.call(
        Some(alice_wallet),
        "sendtoaddress",
        &[receiver_address.to_owned(), 1.0_f64.to_string()],
    )
}

pub fn audit_multi_utxo_spend<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
    funding_utxos: &[Utxo],
) -> LabResult<MultiUtxoAudit> {
    let spend_txid = send_combined_payment(client, alice_wallet, receiver_address)?;
    let transaction = decode_verbose_transaction(client, &spend_txid)?;
    let payment_and_change = identify_payment_and_change(&transaction, receiver_address)?;
    let fee = calculate_fee(&transaction)?;

    Ok(MultiUtxoAudit {
        funding_outpoints: funding_utxos.iter().map(|utxo| utxo.outpoint()).collect(),
        spend_txid,
        spend_input_count: transaction.inputs.len(),
        payment_and_change,
        fee,
    })
}
