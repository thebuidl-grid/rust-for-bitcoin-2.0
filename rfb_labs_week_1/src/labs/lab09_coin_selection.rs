//! Lab 09 — force and audit multi-UTXO coin selection.

use crate::labs::lab06_decode::{
    calculate_fee, decode_verbose_transaction, identify_payment_and_change, input_outpoints,
};
use crate::model::{MultiUtxoAudit, Utxo};
use crate::rpc::{parse_cli_value, RpcClient};
use crate::LabResult;

/// Send three separate 0.4 BTC funding transactions and return their TXIDs.
pub fn create_three_funding_transactions<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    alice_address: &str,
) -> LabResult<Vec<String>> {
    let mut txids = Vec::new();

    for _ in 0..3 {
        let call = client.call(
            Some(miner_wallet),
            "sendtoaddress",
            &[alice_address.to_string(), "0.4".to_string()],
        )?;
        let val = parse_cli_value(&call)?;

        let txid = val.as_str().map(|s| s.to_string()).ok_or_else(|| {
            crate::LabError::Parse("sendtoaddress response is not a string".to_string())
        })?;

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
    let all_utxos = crate::labs::lab04_utxos::list_unspent(client, wallet_name)?;

    let confirmed_for_address = all_utxos
        .into_iter()
        .filter(|u| u.confirmations > 0 && u.address.as_deref() == Some(address))
        .collect();

    Ok(confirmed_for_address)
}

/// Send 1 BTC from Alice and return the new TXID.
pub fn send_combined_payment<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
) -> LabResult<String> {
    let call = client.call(
        Some(alice_wallet),
        "sendtoaddress",
        &[receiver_address.to_string(), "1".to_string()],
    )?;
    let val = parse_cli_value(&call)?;

    val.as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| crate::LabError::Parse("sendtoaddress response is not a string".to_string()))
}

/// Decode Alice's spend and prove that multiple funding UTXOs were combined.
pub fn audit_multi_utxo_spend<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
    funding_utxos: &[Utxo],
) -> LabResult<MultiUtxoAudit> {
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
    let funding_outpoints = funding_utxos
        .iter()
        .map(crate::labs::lab04_utxos::outpoint)
        .collect();

    Ok(MultiUtxoAudit {
        fee,
        payment_and_change,
        funding_outpoints,
        spend_txid,
        spend_input_count,
    })
}
