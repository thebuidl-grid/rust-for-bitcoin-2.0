//! Lab 04 — inspect UTXOs and outpoints.

use serde_json::Value;

use crate::model::{OutPoint, Utxo};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    //     // TODO: call listunspent in wallet context and decode every returned UTXO.
    //     // todo!("Lab 04: list unspent outputs")
    let call = client.call(Some(wallet_name), "listunspent", &[])?;
    let response: Value =
        serde_json::from_str(&call).map_err(|e| LabError::Parse(e.to_string()))?;

    let entries = response
        .as_array()
        .ok_or(LabError::MissingField("listunspent"))?;

    entries
        .iter()
        .map(|entry| {
            let txid = entry["txid"]
                .as_str()
                .ok_or(LabError::MissingField("txid"))?
                .to_string();
            let vout = entry["vout"]
                .as_u64()
                .ok_or(LabError::MissingField("vout"))? as u32;
            let address = entry["address"].as_str().map(|s| s.to_string());
            let script_pub_key = entry["scriptPubKey"]
                .as_str()
                .ok_or(LabError::MissingField("scriptPubKey"))?
                .to_string();
            let amount = entry["amount"]
                .as_f64()
                .ok_or(LabError::MissingField("amount"))?;
            let confirmations = entry["confirmations"]
                .as_u64()
                .ok_or(LabError::MissingField("confirmations"))?;
            let spendable = entry["spendable"]
                .as_bool()
                .ok_or(LabError::MissingField("spendable"))?;

            Ok(Utxo {
                txid,
                vout,
                address,
                script_pub_key,
                amount,
                confirmations,
                spendable,
            })
        })
        .collect()
}

//     Ok(Utxo {
//         txid: required_string(entry, "txid")?,
//         vout: required_f64(entry, "vout")? as u32,
//         address: entry.get("address").and_then(Value::as_str).map(ToOwned::to_owned),
//         script_pub_key: required_string(entry, "scriptPubKey")?,
//         amount: required_f64(entry, "amount")?,
//         confirmations: required_u64(entry, "confirmations")?,
//         spendable: entry.get("spendable").and_then(Value::as_bool).ok_or(LabError::MissingField("spendable"))?,
//     })
// }

// pub fn list_unspent<C: RpcClient>(entry: &Value) -> LabResult<Utxo> {
//     // TODO: call listunspent in wallet context and decode every returned UTXO.
//     // todo!("Lab 04: list unspent outputs")
//     Ok(Utxo {
//         txid: required_string(entry, "txid")?,
//         vout: required_f64(entry, "vout")? as u32,
//         address: entry
//             .get("address")
//             .and_then(Value::as_str)
//             .map(ToOwned::to_owned),
//         script_pub_key: required_string(entry, "scriptPubKey")?,
//         amount: required_f64(entry, "amount")?,
//         confirmations: required_u64(entry, "confirmations")?,
//         spendable: entry
//             .get("spendable")
//             .and_then(Value::as_bool)
//             .ok_or(LabError::MissingField("spendable"))?,
//     })
// }

/// Select one spendable UTXO, preferring the one with the most confirmations.
pub fn select_spendable_utxo(utxos: &[Utxo]) -> Option<Utxo> {
    // TODO: filter by spendable and select deterministically.
    // todo!("Lab 04: select a spendable UTXO")
    utxos
        .iter()
        .filter(|u| u.spendable)
        .max_by_key(|u| u.confirmations)
        .cloned()
}

/// Convert a UTXO into its unique `txid:vout` coordinate.
pub fn outpoint(utxo: &Utxo) -> OutPoint {
    // TODO: return the matching outpoint.
    // todo!("Lab 04: construct an outpoint")
    OutPoint {
        txid: utxo.txid.clone(),
        vout: utxo.vout,
    }
}

/// Sum only the spendable UTXOs.
pub fn sum_spendable_utxos(utxos: &[Utxo]) -> f64 {
    // TODO: ignore non-spendable entries and sum BTC amounts.
    // todo!("Lab 04: calculate spendable wallet balance")
    utxos
        .iter()
        .filter(|utxo| utxo.spendable)
        .map(|utxo| utxo.amount)
        .sum()
}
