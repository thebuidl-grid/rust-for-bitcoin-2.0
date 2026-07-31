//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::RpcClient;
use crate::{LabError, LabResult};

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    let response = client.call(Some(wallet_name), "listunspent", &[])?;
    let json: serde_json::Value = serde_json::from_str(&response)?;
    let entries = json
        .as_array()
        .ok_or_else(|| LabError::Parse("expected a UTXO array".to_string()))?;
    let mut utxos = Vec::with_capacity(entries.len());

    for entry in entries {
        let txid = required_string(entry, "txid")?;
        let vout = required_u32(entry, "vout")?;
        let address = optional_string(entry, "address")?;
        let script_pub_key = required_string(entry, "scriptPubKey")?;
        let amount = required_f64(entry, "amount")?;
        let confirmations = required_u64(entry, "confirmations")?;
        let spendable = entry
            .get("spendable")
            .and_then(serde_json::Value::as_bool)
            .ok_or(LabError::MissingField("spendable"))?;

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
    // filter by spendable and select deterministically.
    utxos
        .iter()
        .filter(|utxo| utxo.spendable)
        .max_by_key(|utxo| utxo.confirmations)
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
    // ignore non-spendable entries and sum BTC amounts.
    utxos
        .iter()
        .filter(|utxo| utxo.spendable)
        .map(|utxo| utxo.amount)
        .sum()
}

fn required_string(value: &serde_json::Value, field: &'static str) -> LabResult<String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .ok_or(LabError::MissingField(field))
}

fn optional_string(value: &serde_json::Value, field: &'static str) -> LabResult<Option<String>> {
    let Some(field_value) = value.get(field) else {
        return Ok(None);
    };
    let field_value = field_value
        .as_str()
        .ok_or_else(|| LabError::Parse(format!("invalid `{field}` field")))?;

    Ok(Some(field_value.to_owned()))
}

fn required_u32(value: &serde_json::Value, field: &'static str) -> LabResult<u32> {
    let number = required_u64(value, field)?;
    u32::try_from(number).map_err(|_| LabError::Parse(format!("`{field}` does not fit in u32")))
}

fn required_u64(value: &serde_json::Value, field: &'static str) -> LabResult<u64> {
    value
        .get(field)
        .and_then(serde_json::Value::as_u64)
        .ok_or(LabError::MissingField(field))
}

fn required_f64(value: &serde_json::Value, field: &'static str) -> LabResult<f64> {
    value
        .get(field)
        .and_then(serde_json::Value::as_f64)
        .ok_or(LabError::MissingField(field))
}
