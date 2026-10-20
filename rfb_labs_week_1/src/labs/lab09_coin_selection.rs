//! Lab 09 — force and audit multi-UTXO coin selection.

use crate::model::{DecodedOutput, MultiUtxoAudit, OutPoint, PaymentAndChange, Utxo};
use crate::rpc::RpcClient;
use crate::LabResult;
use serde::Deserialize;

#[derive(Deserialize)]
struct RawTx {
    vin: Vec<RawVin>,
    vout: Vec<RawVout>,
}

#[derive(Deserialize)]
struct RawVin {
    prevout: Option<RawPrevout>,
}

#[derive(Deserialize)]
struct RawPrevout {
    value: f64,
}

#[derive(Deserialize)]
struct RawVout {
    n: u32,
    value: f64,
    #[serde(rename = "scriptPubKey")]
    script_pub_key: RawScriptPubKey,
}

#[derive(Deserialize)]
struct RawScriptPubKey {
    hex: String,
    address: Option<String>,
}

/// Helper to safely parse string TXIDs returned from RPC client calls
fn parse_txid_response(raw_json: &str) -> String {
    serde_json::from_str::<String>(raw_json)
        .unwrap_or_else(|_| raw_json.trim().trim_matches('"').to_string())
}

/// Send three separate 0.4 BTC funding transactions and return their TXIDs.
pub fn create_three_funding_transactions<C: RpcClient>(
    client: &C,
    miner_wallet: &str,
    alice_address: &str,
) -> LabResult<Vec<String>> {
    let mut txids = Vec::with_capacity(3);
    for _ in 0..3 {
        let json_str = client.call(
            Some(miner_wallet),
            "sendtoaddress",
            &[alice_address.to_string(), "0.4".to_string()],
        )?;
        txids.push(parse_txid_response(&json_str));
    }
    Ok(txids)
}

/// Return confirmed UTXOs belonging to the supplied address.
pub fn confirmed_utxos_for_address<C: RpcClient>(
    client: &C,
    wallet_name: &str,
    address: &str,
) -> LabResult<Vec<Utxo>> {
    let json_str = client.call(Some(wallet_name), "listunspent", &[])?;
    let all_utxos: Vec<Utxo> = serde_json::from_str(&json_str)?;

    let filtered = all_utxos
        .into_iter()
        .filter(|utxo| utxo.confirmations > 0 && utxo.address.as_deref() == Some(address))
        .collect();

    Ok(filtered)
}

/// Send 1 BTC from Alice and return the new TXID.
pub fn send_combined_payment<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
) -> LabResult<String> {
    let json_str = client.call(
        Some(alice_wallet),
        "sendtoaddress",
        &[receiver_address.to_string(), "1".to_string()],
    )?;
    Ok(parse_txid_response(&json_str))
}

/// Decode Alice's spend and prove that multiple funding UTXOs were combined.
pub fn audit_multi_utxo_spend<C: RpcClient>(
    client: &C,
    alice_wallet: &str,
    receiver_address: &str,
    funding_utxos: &[Utxo],
) -> LabResult<MultiUtxoAudit> {
    let spend_txid = send_combined_payment(client, alice_wallet, receiver_address)?;

    let json_str = client.call(None, "getrawtransaction", &[spend_txid.clone(), "2".to_string()])?;
    let raw_tx: RawTx = serde_json::from_str(&json_str)?;

    // Preserve exact txids passed in funding_utxos
 let funding_outpoints = funding_utxos
    .iter()
    .map(|u| OutPoint {
        txid: u.txid.clone(),
        vout: u.vout,
    })
    .collect();

    let total_input: f64 = raw_tx
        .vin
        .iter()
        .filter_map(|vin| vin.prevout.as_ref().map(|p| p.value))
        .sum();

    let mut payment_opt = None;
    let mut change_opt = None;
    let mut total_output = 0.0;

    for vout in raw_tx.vout {
        total_output += vout.value;
        let decoded = DecodedOutput {
            vout: vout.n,
            value: vout.value,
            address: vout.script_pub_key.address.clone(),
            script_pub_key_hex: vout.script_pub_key.hex.clone(),
        };

        if vout.script_pub_key.address.as_deref() == Some(receiver_address) && payment_opt.is_none() {
            payment_opt = Some(decoded);
        } else {
            change_opt = Some(decoded);
        }
    }

    let payment = payment_opt.expect("Payment output not found");

    let fee = (total_input - total_output).max(0.0);
    let fee = (fee * 100_000_000.0).round() / 100_000_000.0;

    Ok(MultiUtxoAudit {
        funding_outpoints,
        spend_txid,
        spend_input_count: raw_tx.vin.len(),
        payment_and_change: PaymentAndChange {
            payment,
            change: change_opt,
        },
        fee,
    })
}