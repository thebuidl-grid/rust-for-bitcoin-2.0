//! A simplified Bitcoin transaction model for the Week 2 assignment.
//!
//! Values are integer satoshis throughout. Nothing here serializes, signs, or
//! broadcasts a real transaction; the point is the ownership and borrowing
//! rules that a transaction's structure makes natural.
//!
//! The public names and signatures are unchanged from the starter crate.
//! `state` is the optional Part 10 extension.

pub mod error;
pub mod state;
pub mod transaction;
pub mod utxo;

pub use error::TransactionError;
pub use state::{InvalidTransition, TransactionLifecycle, TransactionState};
pub use transaction::{
    find_outputs_for_recipient, highest_value_output, BitcoinValue, InputKind, OutPoint,
    OutputType, Transaction, TxOutput,
};
pub use utxo::{select_utxos, Utxo};
