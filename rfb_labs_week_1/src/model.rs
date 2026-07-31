use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkSnapshot {
    pub chain: String,
    pub block_height: u64,
    pub best_block_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletBalances {
    pub trusted: f64,
    pub untrusted_pending: f64,
    pub immature: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoinbaseMaturityReport {
    pub height_after_first_block: u64,
    pub balance_after_first_block: WalletBalances,
    pub premature_spend_error: String,
    pub final_height: u64,
    pub final_balance: WalletBalances,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutPoint {
    pub txid: String,
    pub vout: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Utxo {
    pub txid: String,
    pub vout: u32,
    pub address: Option<String>,
    #[serde(default)] // 👈 Makes missing `label` default to None
    pub label: Option<String>,
    #[serde(rename = "scriptPubKey")]
    pub script_pub_key: String,
    pub amount: f64,
    pub confirmations: u64,
    #[serde(default)] // 👈 Defaults to false if missing
    pub spendable: bool,
    #[serde(default)] // 👈 Fixes "missing field solvable" panic
    pub solvable: bool,
}

impl Utxo {
    pub fn outpoint(&self) -> OutPoint {
        OutPoint {
            txid: self.txid.clone(),
            vout: self.vout,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletTransactionStatus {
    #[serde(default)]
    pub txid: String, // 👈 `serde(default)` defaults missing `txid` to an empty String "" instead of failing!
    pub confirmations: i64,
    pub fee: Option<f64>,
    #[serde(rename = "blockhash")]
    pub block_hash: Option<String>,
    pub amount: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MempoolObservation {
    pub txid: String,
    pub mempool_contains_tx: bool,
    pub sender_status: WalletTransactionStatus,
    pub receiver_balance: WalletBalances,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DecodedInput {
    pub previous_output: OutPoint,
    pub previous_value: f64,
}

impl<'de> Deserialize<'de> for DecodedInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct PrevOut {
            value: f64,
        }

        #[derive(Deserialize)]
        struct RawVin {
            txid: String,
            vout: u32,
            prevout: PrevOut,
        }

        let raw = RawVin::deserialize(deserializer)?;
        Ok(DecodedInput {
            previous_output: OutPoint {
                txid: raw.txid,
                vout: raw.vout,
            },
            previous_value: raw.prevout.value,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DecodedOutput {
    pub vout: u32,
    pub value: f64,
    pub address: Option<String>,
    pub script_pub_key_hex: String,
}

impl<'de> Deserialize<'de> for DecodedOutput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ScriptPubKey {
            hex: String,
            address: Option<String>,
        }

        #[derive(Deserialize)]
        struct RawVout {
            n: u32,
            value: f64,
            #[serde(rename = "scriptPubKey")]
            script_pub_key: ScriptPubKey,
        }

        let raw = RawVout::deserialize(deserializer)?;
        Ok(DecodedOutput {
            vout: raw.n,
            value: raw.value,
            address: raw.script_pub_key.address,
            script_pub_key_hex: raw.script_pub_key.hex,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodedTransaction {
    pub txid: String,
    #[serde(rename = "vin")]
    pub inputs: Vec<DecodedInput>,
    #[serde(rename = "vout")]
    pub outputs: Vec<DecodedOutput>,
    pub vsize: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaymentAndChange {
    pub payment: DecodedOutput,
    pub change: Option<DecodedOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfirmationReport {
    pub txid: String,
    pub block_hash: String,
    pub confirmations: i64,
    pub mempool_is_empty: bool,
    pub transaction_is_in_block: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockHeaderEvidence {
    pub hash: String,
    pub height: u64,
    #[serde(rename = "previousblockhash")]
    pub previous_block_hash: Option<String>,
    #[serde(rename = "merkleroot")]
    pub merkle_root: String,
    pub nonce: u64,
    pub difficulty: f64,
    pub bits: String,
    pub confirmations: i64,
    pub chainwork: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityReport {
    pub header: BlockHeaderEvidence,
    pub confirmations_before: i64,
    pub confirmations_after: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiUtxoAudit {
    pub funding_outpoints: Vec<OutPoint>,
    pub spend_txid: String,
    pub spend_input_count: usize,
    pub payment_and_change: PaymentAndChange,
    pub fee: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainTip {
    pub height: u64,
    pub best_block_hash: String,
    pub chainwork: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForkSnapshot {
    pub node_a: ChainTip,
    pub node_b: ChainTip,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReorgReport {
    pub common_tip_before_split: String,
    pub competing_tips: ForkSnapshot,
    pub final_tips: ForkSnapshot,
    pub converged: bool,
}
