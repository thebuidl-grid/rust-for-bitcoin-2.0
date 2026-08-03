use crate::model::{OutPoint, Utxo};
use crate::rpc::{parse_cli_value, required_f64, required_string, required_u64, RpcClient};
use crate::{LabError, LabResult};
use serde_json::Value;

pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    let raw = client.call(Some(wallet_name), "listunspent", &[])?;
    let value = parse_cli_value(&raw)?;

    value
        .as_array()
        .ok_or_else(|| LabError::Parse("expected a JSON array of UTXOs".to_owned()))?
        .iter()
        .map(decode_utxo)
        .collect()
}

fn decode_utxo(value: &Value) -> LabResult<Utxo> {
    Ok(Utxo {
        txid: required_string(value, "txid")?,
        vout: required_u64(value, "vout")? as u32,
        address: value
            .get("address")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        script_pub_key: required_string(value, "scriptPubKey")?,
        amount: required_f64(value, "amount")?,
        confirmations: required_u64(value, "confirmations")?,
        spendable: value
            .get("spendable")
            .and_then(Value::as_bool)
            .ok_or(LabError::MissingField("spendable"))?,
    })
}

pub fn select_spendable_utxo(utxos: &[Utxo]) -> Option<Utxo> {
    utxos
        .iter()
        .filter(|utxo| utxo.spendable)
        .max_by_key(|utxo| utxo.confirmations)
        .cloned()
}

pub fn outpoint(utxo: &Utxo) -> OutPoint {
    utxo.outpoint()
}

pub fn sum_spendable_utxos(utxos: &[Utxo]) -> f64 {
    utxos
        .iter()
        .filter(|utxo| utxo.spendable)
        .map(|utxo| utxo.amount)
        .sum()
}
