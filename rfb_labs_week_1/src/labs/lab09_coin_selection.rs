//! Lab 09 — force and audit multi-UTXO coin selection.

use crate::model::{DecodedOutput, MultiUtxoAudit, OutPoint, PaymentAndChange, Utxo};
use crate::rpc::RpcClient;
use crate::LabResult;
use serde_json::Value;

/// Send three separate 0.4 BTC funding transactions and return their TXIDs.
pub fn create_three_funding_transactions<C: RpcClient>(
    client: &C,
    _miner_wallet: &str,
    alice_address: &str,
) -> LabResult<Vec<String>> {
    let mut txids = Vec::new();

    for _ in 0..3 {
        let txid = client.call(
            Some("miner"),
            "sendtoaddress",
            &[alice_address.to_owned(), "0.4".to_owned()],
        )?;

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
    let raw = client.call(Some(wallet_name), "listunspent", &[])?;
    let items: Vec<Value> = serde_json::from_str(&raw)?;

    let mut result = Vec::new();

    for item in items {
        if item["address"].as_str() != Some(address) {
            continue;
        }

        if item["confirmations"].as_u64().unwrap_or(0) == 0 {
            continue;
        }

        result.push(Utxo {
            txid: item["txid"].as_str().unwrap_or_default().to_owned(),
            vout: item["vout"].as_u64().unwrap_or(0) as u32,
            address: Some(address.to_owned()),
            script_pub_key: item["scriptPubKey"].as_str().unwrap_or_default().to_owned(),
            amount: item["amount"].as_f64().unwrap_or(0.0),
            confirmations: item["confirmations"].as_u64().unwrap_or(0),
            spendable: item["spendable"].as_bool().unwrap_or(false),
        });
    }

    Ok(result)
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
        &[receiver_address.to_owned(), "1".to_owned()],
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

    let raw = client.call(
        None,
        "getrawtransaction",
        &[spend_txid.clone(), "2".to_owned()],
    )?;

    let tx: Value = serde_json::from_str(&raw)?;

    let funding_outpoints = funding_utxos
        .iter()
        .map(|u| OutPoint {
            txid: u.txid.clone(),
            vout: u.vout,
        })
        .collect();

    let inputs = tx["vin"].as_array().unwrap();
    let outputs = tx["vout"].as_array().unwrap();

    let payment_json = outputs
        .iter()
        .find(|o| o["scriptPubKey"]["address"].as_str() == Some(receiver_address))
        .unwrap();

    let payment = DecodedOutput {
        vout: payment_json["n"].as_u64().unwrap() as u32,
        value: payment_json["value"].as_f64().unwrap(),
        address: Some(receiver_address.to_owned()),
        script_pub_key_hex: payment_json["scriptPubKey"]["hex"]
            .as_str()
            .unwrap()
            .to_owned(),
    };

    let change = outputs
        .iter()
        .find(|o| {
            o["scriptPubKey"]["address"]
                .as_str()
                .unwrap_or("")
                .contains("change")
        })
        .map(|o| DecodedOutput {
            vout: o["n"].as_u64().unwrap() as u32,
            value: o["value"].as_f64().unwrap(),
            address: o["scriptPubKey"]["address"].as_str().map(|s| s.to_owned()),
            script_pub_key_hex: o["scriptPubKey"]["hex"].as_str().unwrap().to_owned(),
        });

    let input_total: f64 = inputs
        .iter()
        .map(|v| v["prevout"]["value"].as_f64().unwrap())
        .sum();

    let output_total: f64 = outputs.iter().map(|o| o["value"].as_f64().unwrap()).sum();

    let fee = ((input_total - output_total) * 100000.0).round() / 100000.0;

    Ok(MultiUtxoAudit {
        funding_outpoints,
        spend_txid,
        spend_input_count: inputs.len(),
        payment_and_change: PaymentAndChange { payment, change },
        fee,
    })
}
