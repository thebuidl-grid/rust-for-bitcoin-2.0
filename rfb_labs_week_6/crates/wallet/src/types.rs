use std::path::PathBuf;

use bitcoin::{Amount, OutPoint, Txid};
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

#[derive(Debug, Default, Serialize)]
pub struct WalletBalance {
    pub confirmed: Amount,
    pub pending: Amount,
    pub immature: Amount,
    pub total: Amount,
}

#[derive(Debug, Serialize)]
pub struct WalletUtxo {
    pub outpoint: OutPoint,
    pub value: Amount,
    pub keychain: Keychain,
    pub confirmed: bool,
}

#[derive(Debug, Serialize)]
pub struct TransactionSummary {
    pub txid: Txid,
    pub sent: Amount,
    pub fee: Amount,
}
