//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::{parse_cli_value, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    let raw = client.call(Some(wallet_name), "listunspent", &[])?;

    let value = parse_cli_value(&raw)?;

    let array = value
        .as_array()
        .ok_or(LabError::Parse("Expected listunspent array".to_owned()))?;

    let mut utxos = Vec::new();

    for item in array {
        utxos.push(Utxo {
            txid: item
                .get("txid")
                .and_then(Value::as_str)
                .ok_or(LabError::MissingField("txid"))?
                .to_owned(),

            vout: item
                .get("vout")
                .and_then(Value::as_u64)
                .ok_or(LabError::MissingField("vout"))? as u32,

            address: item
                .get("address")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),

            script_pub_key: item
                .get("scriptPubKey")
                .and_then(Value::as_str)
                .ok_or(LabError::MissingField("scriptPubKey"))?
                .to_owned(),

            amount: item
                .get("amount")
                .and_then(Value::as_f64)
                .ok_or(LabError::MissingField("amount"))?,

            confirmations: item
                .get("confirmations")
                .and_then(Value::as_u64)
                .ok_or(LabError::MissingField("confirmations"))?,

            spendable: item
                .get("spendable")
                .and_then(Value::as_bool)
                .ok_or(LabError::MissingField("spendable"))?,
        });
    }

    Ok(utxos)
}

/// Select the spendable UTXO with the most confirmations.
pub fn select_spendable_utxo(utxos: &[Utxo]) -> Option<Utxo> {
    utxos
        .iter()
        .filter(|utxo| utxo.spendable)
        .max_by_key(|utxo| utxo.confirmations)
        .cloned()
}

/// Convert a UTXO into its unique txid:vout coordinate.
pub fn outpoint(utxo: &Utxo) -> OutPoint {
    OutPoint {
        txid: utxo.txid.clone(),
        vout: utxo.vout,
    }
}

/// Sum only spendable UTXOs.
pub fn sum_spendable_utxos(utxos: &[Utxo]) -> f64 {
    utxos
        .iter()
        .filter(|utxo| utxo.spendable)
        .map(|utxo| utxo.amount)
        .sum()
}
