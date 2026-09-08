use std::fmt;

use crate::state::TransactionState;

/// Expected failures produced by transaction validation and coin selection.
#[derive(Debug, PartialEq, Eq)]
pub enum TransactionError {
    NoInputs,
    NoOutputs,
    ZeroValueOutput,
    OutputsExceedInputs {
        total_inputs: u64,
        total_outputs: u64,
    },
    CoinbaseMixedWithRegularInputs,
    MultipleCoinbaseInputs,
    InvalidTxid,
    InsufficientFunds {
        available: u64,
        required: u64,
    },
    InvalidStateTransition {
        from: TransactionState,
        to: TransactionState,
    },
}

impl fmt::Display for TransactionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(Part 4): return a useful message for every error variant.
        // todo!("implement Display for TransactionError")
        match self {
            TransactionError::NoInputs => write!(formatter, "no inputs"),
            TransactionError::InsufficientFunds {
                available,
                required,
            } => {
                write!(
                    formatter,
                    "{} available funds, {} required",
                    available, required
                )
            }
            TransactionError::NoOutputs => write!(formatter, "no outputs"),
            TransactionError::ZeroValueOutput => write!(formatter, "zero value output"),
            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => {
                write!(
                    formatter,
                    "{} outputs exceed {} total inputs, check transaction",
                    total_inputs, total_outputs
                )
            }
            TransactionError::CoinbaseMixedWithRegularInputs => {
                write!(
                    formatter,
                    "Mixing coinbase and regular inputs are not allowed"
                )
            }
            TransactionError::InvalidTxid => write!(formatter, "invalid txid"),
            TransactionError::MultipleCoinbaseInputs => {
                write!(formatter, "Multiple coinbase inputs are not allowed")
            }
            TransactionError::InvalidStateTransition { from, to } => {
                write!(
                    formatter,
                    "invalid state transition from {from:?} to {to:?}"
                )
            }
        }
    }
}

impl std::error::Error for TransactionError {}
