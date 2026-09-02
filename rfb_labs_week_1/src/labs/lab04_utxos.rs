//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::{parse_cli_value, required_f64, required_string, RpcClient};
use crate::{LabError, LabResult};

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    // 1. Call `listunspent` within the context of `wallet_name`
    let raw = client.call(Some(wallet_name), "listunspent", &[])?;
    let value = parse_cli_value(&raw)?;

    // 2. Decode JSON array into Vec<Utxo>
    let array = value
        .as_array()
        .ok_or_else(|| LabError::Parse("listunspent did not return an array".to_owned()))?;

    let mut utxos = Vec::with_capacity(array.len());

    for item in array {
        let txid = required_string(item, "txid")?;
        
        let vout = item["vout"]
            .as_u64()
            .ok_or_else(|| LabError::Parse("listunspent item missing numeric 'vout'".to_owned()))?
            as u32;

        let address = item["address"].as_str().map(ToOwned::to_owned);
        let script_pub_key = required_string(item, "scriptPubKey")?;
        let amount = required_f64(item, "amount")?;

        let confirmations = item["confirmations"]
            .as_u64()
            .ok_or_else(|| LabError::Parse("listunspent item missing numeric 'confirmations'".to_owned()))?;

        let spendable = item["spendable"]
            .as_bool()
            .ok_or_else(|| LabError::Parse("listunspent item missing boolean 'spendable'".to_owned()))?;

        utxos.push(Utxo {
            txid,
            vout,
            address,
            script_pub_key,
            amount,
            confirmations,
            spendable,
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