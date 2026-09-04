//! Week 3 — Understanding Bitcoin Data.
//!
//! A from-scratch raw Bitcoin transaction parser and decoder. No Bitcoin
//! library is used for the wire format: [`transaction`] implements BIP144
//! (SegWit) and the classic layout byte by byte, [`script`] disassembles
//! Script, and [`decode`] projects the result into a
//! `decoderawtransaction`-style JSON view.

pub mod decode;
pub mod error;
pub mod reader;
pub mod script;
pub mod transaction;

pub use decode::DecodedTransaction;
pub use error::{ParseError, Result};
pub use transaction::{OutPoint, Transaction, TxIn, TxOut};

/// Parse a hex-encoded transaction into the structured model.
pub fn parse_transaction_hex(raw_hex: &str) -> Result<Transaction> {
    let cleaned: String = raw_hex.split_whitespace().collect();
    let bytes = hex::decode(&cleaned).map_err(|e| ParseError::BadHex(e.to_string()))?;
    Transaction::parse(&bytes)
}

/// Parse and then project into the JSON-friendly decoded view.
pub fn decode_transaction_hex(raw_hex: &str) -> Result<DecodedTransaction> {
    Ok(DecodedTransaction::from_transaction(
        &parse_transaction_hex(raw_hex)?,
    ))
}
