//! Lab 09 — force and audit multi-UTXO coin selection.

use crate::labs::lab04_utxos::list_unspent;
use crate::labs::lab05_mempool::send_btc;
use crate::labs::lab06_decode::{
    calculate_fee, decode_verbose_transaction, identify_payment_and_change, input_outpoints,
};
use crate::model::{MultiUtxoAudit, Utxo};
use crate::rpc::RpcClient;
use crate::{LabError, LabResult};

/// Send three separate 0.4 BTC funding transactions and return their TXIDs.
pub fn create_three_funding_transactions<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    alice_address: &str,
) -> LabResult<Vec<String>> {
    (0..3)
        .map(|_| send_btc(client, miner_wallet, alice_address, 0.4))
        .collect()
}

/// Return confirmed UTXOs belonging to the supplied address.
pub fn confirmed_utxos_for_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<Vec<Utxo>> {
    Ok(list_unspent(client, wallet_name)?
        .into_iter()
        .filter(|utxo| utxo.confirmations > 0 && utxo.address.as_deref() == Some(address))
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
    let transaction = decode_verbose_transaction(client, &spend_txid)?;
    let funding_outpoints: Vec<_> = funding_utxos.iter().map(Utxo::outpoint).collect();
    let spent_outpoints = input_outpoints(&transaction);

    if spent_outpoints.len() < 2 {
        return Err(LabError::Parse(
            "combined payment did not use multiple inputs".into(),
        ));
    }
    if spent_outpoints
        .iter()
        .any(|outpoint| !funding_outpoints.contains(outpoint))
    {
        return Err(LabError::Parse(
            "combined payment used an unexpected funding outpoint".into(),
        ));
    }

    Ok(MultiUtxoAudit {
        funding_outpoints,
        spend_txid,
        spend_input_count: transaction.inputs.len(),
        payment_and_change: identify_payment_and_change(&transaction, receiver_address)?,
        fee: calculate_fee(&transaction)?,
    })
}
