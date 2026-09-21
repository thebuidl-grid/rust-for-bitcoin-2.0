//! Lab 09 — force and audit multi-UTXO coin selection.

use crate::labs::lab06_decode::{
    calculate_fee, decode_verbose_transaction, identify_payment_and_change,
};
use crate::model::{MultiUtxoAudit, OutPoint, Utxo};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};


/// Send three separate 0.4 BTC funding transactions and return their TXIDs.
pub fn create_three_funding_transactions<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    alice_address: &str,
) -> LabResult<Vec<String>> {
    // TODO: call sendtoaddress three times, each for 0.4 BTC.
    let mut txids = Vec::with_capacity(3);

    for _ in 0..3 {
        let raw = client.call(
            Some(miner_wallet),
            "sendtoaddress",
            &[alice_address.to_string(), "0.4".to_string()],
        )?;
        let val = parse_cli_value(&raw)?;

        let txid = val.as_str().ok_or_else(|| {
            LabError::Parse("expected string response from sendtoaddress".to_string())
        })?;

        txids.push(txid.to_string());
    }

    Ok(txids)
}

/// Return confirmed UTXOs belonging to the supplied address.
pub fn confirmed_utxos_for_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<Vec<Utxo>> {
    // TODO: call listunspent and retain confirmed outputs for this address.
        let raw = client.call(Some(wallet_name), "listunspent", &[])?;
    let val = parse_cli_value(&raw)?;

    let utxo_array = val
        .as_array()
        .ok_or_else(|| LabError::Parse("expected array from listunspent".to_string()))?;

    let mut result = Vec::new();
    for utxo_val in utxo_array {
        let utxo_address = utxo_val
            .get("address")
            .and_then(|a| a.as_str())
            .map(ToOwned::to_owned);

        // Filter only UTXOs matching Alice's address
        if utxo_address.as_deref() == Some(address) {
            let txid = required_string(utxo_val, "txid")?;
            let vout = required_u64(utxo_val, "vout")? as u32;
            let script_pub_key = required_string(utxo_val, "scriptPubKey")?;
            let amount = required_f64(utxo_val, "amount")?;
            let confirmations = required_u64(utxo_val, "confirmations")?;
            let spendable = utxo_val
                .get("spendable")
                .and_then(|s| s.as_bool())
                .unwrap_or(true);

            result.push(Utxo {
                txid,
                vout,
                address: utxo_address,
                script_pub_key,
                amount,
                confirmations,
                spendable,
            });
        }
    }

    Ok(result)
}


/// Send 1 BTC from Alice and return the new TXID.
pub fn send_combined_payment<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
) -> LabResult<String> {
    // TODO: call sendtoaddress for 1 BTC.
    let raw = client.call(
        Some(alice_wallet),
        "sendtoaddress",
        &[receiver_address.to_string(), "1".to_string()],
    )?;
    let val = parse_cli_value(&raw)?;

    let txid = val.as_str().ok_or_else(|| {
        LabError::Parse("expected string response from sendtoaddress".to_string())
    })?;

    Ok(txid.to_string())
}

/// Decode Alice's spend and prove that multiple funding UTXOs were combined.
pub fn audit_multi_utxo_spend<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
    funding_utxos: &[Utxo],
) -> LabResult<MultiUtxoAudit> {
let funding_outpoints: Vec<OutPoint> = funding_utxos.iter().map(|u| u.outpoint()).collect();

    // 2. Send 1 BTC payment requiring multi-UTXO selection
    let spend_txid = send_combined_payment(client, alice_wallet, receiver_address)?;

    // 3. Reuse Lab 06 helper to decode transaction verbosely
    let decoded_tx = decode_verbose_transaction(client, &spend_txid)?;

    // 4. Extract total inputs used
    let spend_input_count = decoded_tx.inputs.len();

    // 5. Identify payment and change outputs
    let payment_and_change = identify_payment_and_change(&decoded_tx, receiver_address)?;

    // 6. Calculate fee and round to 8 decimal places (satoshis) to prevent f64 drift
    let fee = (calculate_fee(&decoded_tx)? * 1_0000_0000.0).round() / 1_0000_0000.0;

    Ok(MultiUtxoAudit {
        funding_outpoints,
        spend_txid,
        spend_input_count,
        payment_and_change,
        fee,
    })
}
