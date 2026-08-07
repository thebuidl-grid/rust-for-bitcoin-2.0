//! Starter library for the Week 2 Bitcoin transaction assignment.

pub mod error;
pub mod state;
pub mod transaction;
pub mod utxo;

pub use error::TransactionError;
pub use transaction::{
    find_outputs_for_recipient, highest_value_output, BitcoinValue, InputKind, OutPoint,
    OutputType, Transaction, TxOutput,
};
pub use utxo::{select_utxos, Utxo};

pub use state::{TrackedTransaction, TransactionState};
