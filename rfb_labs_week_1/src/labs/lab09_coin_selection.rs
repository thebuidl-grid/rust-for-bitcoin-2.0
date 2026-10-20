//! Lab 09 — force and audit multi-UTXO coin selection.

use crate::labs::lab06_decode::{
    calculate_fee, decode_verbose_transaction, identify_payment_and_change,
};
use crate::model::{MultiUtxoAudit, Utxo};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};

/// Send three separate 0.4 BTC funding transactions and return their TXIDs.
pub fn create_three_funding_transactions<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    alice_address: &str,
) -> LabResult<Vec<String>> {
    let mut txids = Vec::new();

    for _ in 0..3 {
        let response = client.call(
            Some(miner_wallet),
            "sendtoaddress",
            &[alice_address.to_string(), "0.4".to_string()],
        )?;

        txids.push(response.trim_matches('"').to_string());
    }

    Ok(txids)
}

/// Return confirmed UTXOs belonging to the supplied address.
pub fn confirmed_utxos_for_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<Vec<Utxo>> {
    let response = client.call(Some(wallet_name), "listunspent", &[])?;
    let value = parse_cli_value(&response)?;

    let mut utxos = Vec::new();

    for item in value.as_array().unwrap() {
        let utxo = Utxo {
            txid: required_string(item, "txid")?,
            vout: required_u64(item, "vout")? as u32,
            address: item["address"].as_str().map(String::from),
            script_pub_key: required_string(item, "scriptPubKey")?,
            amount: required_f64(item, "amount")?,
            confirmations: required_u64(item, "confirmations")?,
            spendable: item["spendable"]
                .as_bool()
                .ok_or(LabError::Parse("expected boolean".into()))?,
        };

        if utxo.confirmations > 0
            && utxo.address.as_deref() == Some(address)
        {
            utxos.push(utxo);
        }
    }

    Ok(utxos)
}

/// Send 1 BTC from Alice and return the new TXID.
pub fn send_combined_payment<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
) -> LabResult<String> {
    let response = client.call(
        Some(alice_wallet),
        "sendtoaddress",
        &[receiver_address.to_string(), "1".to_string()],
    )?;

    Ok(response.trim_matches('"').to_string())
}

/// Decode Alice's spend and prove that multiple funding UTXOs were combined.
pub fn audit_multi_utxo_spend<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
    funding_utxos: &[Utxo],
) -> LabResult<MultiUtxoAudit> {
    let txid = send_combined_payment(client, alice_wallet, receiver_address)?;

    let tx = decode_verbose_transaction(client, &txid)?;
    let payment_and_change = identify_payment_and_change(&tx, receiver_address)?;
   let fee = (calculate_fee(&tx)? * 100_000_000.0).round() / 100_000_000.0;

    Ok(MultiUtxoAudit {
        funding_outpoints: funding_utxos.iter().map(Utxo::outpoint).collect(),
        spend_txid: txid,
        spend_input_count: tx.inputs.len(),
        payment_and_change,
        fee,
    })
}