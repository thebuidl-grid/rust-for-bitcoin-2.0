//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::{parse_cli_value, RpcClient};
use crate::LabResult;
use serde::Deserialize;

#[derive(Deserialize)]
struct RpcUtxo {
    txid: String,
    vout: u32,
    address: Option<String>,
    #[serde(rename = "scriptPubKey")]
    script_pub_key: String,
    amount: f64,
    confirmations: u64,
    spendable: bool,
}

impl From<RpcUtxo> for Utxo {
    fn from(value: RpcUtxo) -> Self {
        Self {
            txid: value.txid,
            vout: value.vout,
            address: value.address,
            script_pub_key: value.script_pub_key,
            amount: value.amount,
            confirmations: value.confirmations,
            spendable: value.spendable,
        }
    }
}

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    let response = client.call(Some(wallet_name), "listunspent", &[])?;
    let entries: Vec<RpcUtxo> = serde_json::from_value(parse_cli_value(&response)?)?;
    Ok(entries.into_iter().map(Into::into).collect())
}

/// Select one spendable UTXO, preferring the one with the most confirmations.
pub fn select_spendable_utxo(utxos: &[Utxo]) -> Option<Utxo> {
    utxos
        .iter()
        .filter(|utxo| utxo.spendable)
        .min_by(|left, right| {
            right
                .confirmations
                .cmp(&left.confirmations)
                .then_with(|| left.txid.cmp(&right.txid))
                .then_with(|| left.vout.cmp(&right.vout))
        })
        .cloned()
}

/// Convert a UTXO into its unique `txid:vout` coordinate.
pub fn outpoint(utxo: &Utxo) -> OutPoint {
    utxo.outpoint()
}

/// Sum only the spendable UTXOs.
pub fn sum_spendable_utxos(utxos: &[Utxo]) -> f64 {
    utxos
        .iter()
        .filter(|utxo| utxo.spendable)
        .map(|utxo| utxo.amount)
        .sum()
}
