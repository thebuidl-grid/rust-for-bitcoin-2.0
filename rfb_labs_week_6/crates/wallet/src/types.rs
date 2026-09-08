use std::path::PathBuf;

use bitcoin::{Amount, BlockHash, OutPoint, Txid};
use serde::Serialize;

#[derive(Debug)]
pub struct WalletInitialization {
    pub database_path: PathBuf,
    pub network: bitcoin::Network,
    pub recovery_phrase: Option<String>,
    pub external_descriptor: String,
    pub internal_descriptor: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Keychain {
    External,
    Internal,
}

#[derive(Debug, Serialize)]
pub struct DerivedAddress {
    pub address: String,
    pub keychain: Keychain,
    pub derivation_index: u32,
}

#[derive(Debug, Serialize)]
pub struct WalletSync {
    pub blocks_applied: usize,
    pub mempool_transactions: usize,
    pub evicted_transactions: usize,
    pub tip_height: u32,
    pub tip_hash: BlockHash,
}

#[derive(Debug, Default, Serialize)]
pub struct WalletBalance {
    pub confirmed: Amount,
    pub trusted_pending: Amount,
    pub untrusted_pending: Amount,
    pub pending: Amount,
    pub immature: Amount,
    pub spendable: Amount,
    pub total: Amount,
}

#[derive(Debug, Serialize)]
pub struct WalletUtxo {
    pub outpoint: OutPoint,
    pub value: Amount,
    pub keychain: Keychain,
    pub derivation_index: u32,
    pub confirmation_height: Option<u32>,
    pub confirmed: bool,
    pub coinbase: bool,
    pub mature: bool,
    pub locked: bool,
    pub spendable: bool,
}

#[derive(Debug, Serialize)]
pub struct TransactionSummary {
    pub txid: Txid,
    pub sent: Amount,
    pub fee: Amount,
}
