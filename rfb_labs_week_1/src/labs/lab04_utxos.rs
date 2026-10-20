//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::{
    parse_cli_value, required_f64, required_string, required_u64, RpcClient,
};use crate::LabResult;

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    let response = client.call(Some(wallet_name), "listunspent", &[])?;
    let value = parse_cli_value(&response)?;

    let mut utxos = Vec::new();

    for item in value.as_array().unwrap() {
        utxos.push(Utxo {
            txid: required_string(item, "txid")?,
            vout: required_u64(item, "vout")? as u32,
            address: item["address"].as_str().map(str::to_owned),
            script_pub_key: required_string(item, "scriptPubKey")?,
            amount: required_f64(item, "amount")?,
            confirmations: required_u64(item, "confirmations")?,
            spendable: item["spendable"]
    .as_bool()
    .ok_or(crate::LabError::Parse("missing spendable".to_string()))?,
        });
    }

    Ok(utxos)
}

/// Select one spendable UTXO, preferring the one with the most confirmations.
pub fn select_spendable_utxo(utxos: &[Utxo]) -> Option<Utxo> {
    utxos
        .iter()
        .filter(|u| u.spendable)
        .max_by_key(|u| u.confirmations)
        .cloned()
}

/// Convert a UTXO into its unique `txid:vout` coordinate.
pub fn outpoint(utxo: &Utxo) -> OutPoint {
    OutPoint {
        txid: utxo.txid.clone(),
        vout: utxo.vout,
    }
}

/// Sum only the spendable UTXOs.
pub fn sum_spendable_utxos(utxos: &[Utxo]) -> f64 {
    utxos
        .iter()
        .filter(|u| u.spendable)
        .map(|u| u.amount)
        .sum()
}