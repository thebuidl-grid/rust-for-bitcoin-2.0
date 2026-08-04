//! Lab 09 — force and audit multi-UTXO coin selection.

use crate::labs::lab04_utxos::list_unspent;
use crate::labs::lab05_mempool::send_btc;
use crate::labs::lab06_decode::{
    calculate_fee, decode_verbose_transaction, identify_payment_and_change,
};
use crate::model::{MultiUtxoAudit, OutPoint, Utxo};
use crate::rpc::RpcClient;
use crate::LabResult;

/// Send three separate 0.4 BTC funding transactions and return their TXIDs.
pub fn create_three_funding_transactions<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    alice_address: &str,
) -> LabResult<Vec<String>> {
    let mut txids = Vec::new();
    for _ in 0..3 {
        let txid = send_btc(client, miner_wallet, alice_address, 0.4)?;
        txids.push(txid);
    }
    Ok(txids)
}

/// Return confirmed UTXOs belonging to the supplied address.
pub fn confirmed_utxos_for_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<Vec<Utxo>> {
    let utxos = list_unspent(client, wallet_name)?;

    Ok(utxos
        .into_iter()
        .filter(|utxo| {
            utxo.confirmations > 0 && utxo.address.as_ref() == Some(&address.to_string())
        })
        .collect())
}

/// Send 1 BTC from Alice and return the new TXID.
pub fn send_combined_payment<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
) -> LabResult<String> {
    send_btc(client, alice_wallet, receiver_address, 1.0)
}

/// Decode Alice's spend and prove that multiple funding UTXOs were combined.
pub fn audit_multi_utxo_spend<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
    funding_utxos: &[Utxo],
) -> LabResult<MultiUtxoAudit> {
      let spend_txid = send_combined_payment(client, alice_wallet, receiver_address)?;
    let decoded_tx = decode_verbose_transaction(client, &spend_txid)?;
    let payment_and_change = identify_payment_and_change(&decoded_tx, receiver_address)?;
    let raw_fee = calculate_fee(&decoded_tx)?;

    // Round to 8 decimal places (satoshi precision)
    let fee = (raw_fee * 100_000_000.0).round() / 100_000_000.0;

    let funding_outpoints: Vec<OutPoint> =
        funding_utxos.iter().map(|utxo| utxo.outpoint()).collect();

    Ok(MultiUtxoAudit {
        funding_outpoints,
        spend_txid,
        spend_input_count: decoded_tx.inputs.len(),
        payment_and_change,
        fee,
    })
}
