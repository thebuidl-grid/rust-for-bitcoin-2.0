//! A JSON-friendly projection of [`Transaction`], shaped to resemble Bitcoin
//! Core's `decoderawtransaction` output.

use serde::Serialize;

use crate::script::{classify, to_asm};
use crate::transaction::Transaction;

#[derive(Debug, Clone, Serialize)]
pub struct DecodedTransaction {
    pub txid: String,
    /// Witness txid (`wtxid`). Equal to `txid` for non-SegWit transactions.
    pub hash: String,
    pub version: i32,
    /// Full serialized size in bytes (Core's `size`).
    pub size: usize,
    /// Virtual size, `ceil(weight / 4)`.
    pub vsize: usize,
    /// BIP141 weight units.
    pub weight: usize,
    pub locktime: u32,
    /// True when the SegWit marker/flag were present on the wire.
    pub segwit: bool,
    pub vin: Vec<DecodedInput>,
    pub vout: Vec<DecodedOutput>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DecodedInput {
    /// Present only for a coinbase input; holds the raw coinbase script hex.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coinbase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub txid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vout: Option<u32>,
    #[serde(rename = "scriptSig", skip_serializing_if = "Option::is_none")]
    pub script_sig: Option<ScriptView>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub txinwitness: Vec<String>,
    pub sequence: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DecodedOutput {
    /// Amount in satoshis (exact).
    pub value_sat: u64,
    /// Amount in BTC as a fixed 8-decimal string (no float rounding).
    pub value_btc: String,
    pub n: usize,
    #[serde(rename = "scriptPubKey")]
    pub script_pubkey: ScriptPubKeyView,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScriptView {
    pub asm: String,
    pub hex: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScriptPubKeyView {
    pub asm: String,
    pub hex: String,
    #[serde(rename = "type")]
    pub script_type: &'static str,
}

fn sats_to_btc_string(sats: u64) -> String {
    format!("{}.{:08}", sats / 100_000_000, sats % 100_000_000)
}

impl DecodedTransaction {
    pub fn from_transaction(tx: &Transaction) -> Self {
        let vin = tx
            .inputs
            .iter()
            .map(|input| {
                let witness: Vec<String> = input.witness.iter().map(hex::encode).collect();

                if input.previous_output.is_coinbase() {
                    DecodedInput {
                        coinbase: Some(hex::encode(&input.script_sig)),
                        txid: None,
                        vout: None,
                        script_sig: None,
                        txinwitness: witness,
                        sequence: input.sequence,
                    }
                } else {
                    DecodedInput {
                        coinbase: None,
                        txid: Some(input.previous_output.txid_hex()),
                        vout: Some(input.previous_output.vout),
                        script_sig: Some(ScriptView {
                            asm: to_asm(&input.script_sig),
                            hex: hex::encode(&input.script_sig),
                        }),
                        txinwitness: witness,
                        sequence: input.sequence,
                    }
                }
            })
            .collect();

        let vout = tx
            .outputs
            .iter()
            .enumerate()
            .map(|(n, output)| DecodedOutput {
                value_sat: output.value,
                value_btc: sats_to_btc_string(output.value),
                n,
                script_pubkey: ScriptPubKeyView {
                    asm: to_asm(&output.script_pubkey),
                    hex: hex::encode(&output.script_pubkey),
                    script_type: classify(&output.script_pubkey),
                },
            })
            .collect();

        Self {
            txid: tx.txid(),
            hash: tx.wtxid(),
            version: tx.version,
            size: tx.total_size(),
            vsize: tx.vsize(),
            weight: tx.weight(),
            locktime: tx.lock_time,
            segwit: tx.segwit,
            vin,
            vout,
        }
    }
}
