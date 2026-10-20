//! Lab 09 — force and audit multi-UTXO coin selection.


use crate::labs::lab06_decode::{
    calculate_fee, decode_verbose_transaction, identify_payment_and_change, input_outpoints,
};

use crate::model::{MultiUtxoAudit, Utxo};
use crate::rpc::RpcClient;
use crate::{LabResult, LabError};
use serde::Deserialize;

/// Send three separate 0.4 BTC funding transactions and return their TXIDs.
pub fn create_three_funding_transactions<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    alice_address: &str,
) -> LabResult<Vec<String>> {
    // TODO: call sendtoaddress three times, each for 0.4 BTC.
    // todo!("Lab 09: create three separate funding transactions")
    let mut txids = Vec::with_capacity(3);

    for _ in 0..3 {
        let raw = client.call(
            Some(miner_wallet),
            "sendtoaddress",
            &[alice_address.to_string(), "0.4".to_string()],
        )?;
        txids.push(raw.trim().trim_matches('"').to_string());
    }

    Ok(txids)
}

#[derive(Deserialize)]
struct RawUnspent {
    txid: String,
    vout: u32,
    address: Option<String>,
    #[serde(rename = "scriptPubKey")]
    script_pub_key: String,
    amount: f64,
    confirmations: u64,
    spendable: bool,
}
/// Return confirmed UTXOs belonging to the supplied address.
pub fn confirmed_utxos_for_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<Vec<Utxo>> {
    // TODO: call listunspent and retain confirmed outputs for this address.
    // todo!("Lab 09: locate Alice's confirmed UTXOs")
    let raw = client.call(Some(wallet_name), "listunspent", &[])?;
    let entries: Vec<RawUnspent> =
        serde_json::from_str(&raw).map_err(|e| LabError::Parse(e.to_string()))?;

    let utxos = entries
        .into_iter()
        .filter(|entry| entry.confirmations > 0 && entry.address.as_deref() == Some(address))
        .map(|entry| Utxo {
            txid: entry.txid,
            vout: entry.vout,
            address: entry.address,
            script_pub_key: entry.script_pub_key,
            amount: entry.amount,
            confirmations: entry.confirmations,
            spendable: entry.spendable,
        })
        .collect();

    Ok(utxos)
}

/// Send 1 BTC from Alice and return the new TXID.
pub fn send_combined_payment<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
) -> LabResult<String> {
    // TODO: call sendtoaddress for 1 BTC.
    // todo!("Lab 09: create a spend requiring multiple inputs")
    let raw = client.call(
        Some(alice_wallet),
        "sendtoaddress",
        &[receiver_address.to_string(), "1".to_string()],
    )?;

    Ok(raw.trim().trim_matches('"').to_string())
}

/// Decode Alice's spend and prove that multiple funding UTXOs were combined.
pub fn audit_multi_utxo_spend<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
    funding_utxos: &[Utxo],
) -> LabResult<MultiUtxoAudit> {
    // TODO:
    // 1. Send the 1 BTC payment.
    // 2. Reuse Lab 06 to decode it.
    // 3. Identify payment and change.
    // 4. Calculate fee and input count.
    // 5. Record the funding outpoints.
    // todo!("Lab 09: audit multi-UTXO coin selection")
    // 1. Send the 1 BTC payment.
    let spend_txid = send_combined_payment(client, alice_wallet, receiver_address)?;

    // 2. Reuse Lab 06 to decode it.
    let transaction = decode_verbose_transaction(client, &spend_txid)?;

    // 3. Identify payment and change.
    let payment_and_change = identify_payment_and_change(&transaction, receiver_address)?;

    // 4. Calculate fee and input count.
    let fee = calculate_fee(&transaction)?;
    let spend_input_count = transaction.inputs.len();

    // 5. Record the funding outpoints.
    let funding_outpoints = input_outpoints(&transaction);

    let _ = funding_utxos; // available for cross-checking against funding_outpoints if needed

    Ok(MultiUtxoAudit {
        funding_outpoints,
        spend_txid,
        spend_input_count,
        payment_and_change,
        fee,
    })
}
