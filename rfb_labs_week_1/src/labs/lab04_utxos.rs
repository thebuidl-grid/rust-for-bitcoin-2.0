//! Lab 04 — inspect UTXOs and outpoints.

use crate::model::{OutPoint, Utxo};
use crate::rpc::RpcClient;
use crate::LabResult;

use crate::rpc::parse_cli_value;
use crate::LabError;
use serde::Deserialize;

#[derive(Deserialize)]
struct RawUtxo {
    txid: String,
    vout: u32,
    address: Option<String>,
    #[serde(rename = "scriptPubKey")]
    script_pub_key: String,
    amount: f64,
    confirmations: u64,
    spendable: bool,
}

impl From<RawUtxo> for Utxo {
    fn from(raw: RawUtxo) -> Self {
        Utxo {
            txid: raw.txid,
            vout: raw.vout,
            address: raw.address,
            script_pub_key: raw.script_pub_key,
            amount: raw.amount,
            confirmations: raw.confirmations,
            spendable: raw.spendable,
        }
    }
}

/// Return all UTXOs tracked by the selected wallet.
pub fn list_unspent<C: RpcClient>(client: &C, wallet_name: &str) -> LabResult<Vec<Utxo>> {
    let raw = client.call(Some(wallet_name), "listunspent", &[])?;
    let json = parse_cli_value(&raw)?;
    let raw_utxos: Vec<RawUtxo> = serde_json::from_value(json)?;
    Ok(raw_utxos.into_iter().map(Utxo::from).collect())
}

/// Select one spendable UTXO, preferring the one with the most confirmations.
pub fn select_spendable_utxo(utxos: &[Utxo]) -> Option<Utxo> {
    utxos.iter()
        .filter(|utxo| utxo.spendable)
        .max_by_key(|utxo| utxo.confirmations)
        .cloned()
}

/// Convert a UTXO into its unique `txid:vout` coordinate.
pub fn outpoint(utxo: &Utxo) -> OutPoint {
    utxo.outpoint()
}

/// Sum only the spendable UTXOs.
pub fn sum_spendable_utxos(utxos: &[Utxo]) -> f64 {
    utxos.iter()
        .filter(|utxo| utxo.spendable)
        .map(|utxo| utxo.amount)
        .sum()
}
