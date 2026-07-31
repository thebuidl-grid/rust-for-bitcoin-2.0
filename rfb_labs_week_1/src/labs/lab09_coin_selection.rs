//! Lab 09 — force and audit multi-UTXO coin selection.

use crate::labs::lab04_utxos::list_unspent;
use crate::labs::lab06_decode::{
    calculate_fee, decode_verbose_transaction, identify_payment_and_change,
};
use crate::model::{MultiUtxoAudit, Utxo};
use crate::rpc::RpcClient;
use crate::LabResult;

/// Send three separate 0.4 BTC funding transactions and return their TXIDs.
pub fn create_three_funding_transactions<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    alice_address: &str,
) -> LabResult<Vec<String>> {
    let mut transaction_ids = Vec::with_capacity(3);

    for _ in 0..3 {
        let txid = client.call(
            Some(miner_wallet),
            "sendtoaddress",
            &[alice_address.to_string(), "0.4".to_string()],
        )?;
        transaction_ids.push(txid);
    }

    Ok(transaction_ids)
}

/// Return confirmed UTXOs belonging to the supplied address.
pub fn confirmed_utxos_for_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<Vec<Utxo>> {
    let wallet_utxos = list_unspent(client, wallet_name)?;
    let mut confirmed_utxos = Vec::new();

    for utxo in wallet_utxos {
        let belongs_to_address = utxo.address.as_deref() == Some(address);
        if belongs_to_address && utxo.confirmations > 0 {
            confirmed_utxos.push(utxo);
        }
    }

    Ok(confirmed_utxos)
}

/// Send 1 BTC from Alice and return the new TXID.
pub fn send_combined_payment<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
) -> LabResult<String> {
    client.call(
        Some(alice_wallet),
        "sendtoaddress",
        &[receiver_address.to_string(), "1".to_string()],
    )
}

/// Decode Alice's spend and prove that multiple funding UTXOs were combined.
pub fn audit_multi_utxo_spend<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
    funding_utxos: &[Utxo],
) -> LabResult<MultiUtxoAudit> {
    let spend_txid = send_combined_payment(client, alice_wallet, receiver_address)?;
    let decoded_spend = decode_verbose_transaction(client, &spend_txid)?;
    let payment_and_change = identify_payment_and_change(&decoded_spend, receiver_address)?;
    let fee = calculate_fee(&decoded_spend)?;

    let mut funding_outpoints = Vec::with_capacity(funding_utxos.len());
    for utxo in funding_utxos {
        funding_outpoints.push(utxo.outpoint());
    }

    Ok(MultiUtxoAudit {
        funding_outpoints,
        spend_txid,
        spend_input_count: decoded_spend.inputs.len(),
        payment_and_change,
        fee,
    })
}
