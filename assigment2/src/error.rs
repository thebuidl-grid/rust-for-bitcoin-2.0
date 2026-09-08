use std::fmt;

use crate::transaction::{OutPoint, TransactionStatus};

/// Errors that can arise while building, validating, or spending transactions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionError {
    EmptyInputs,
    EmptyOutputs,
    ZeroValueOutput,
    DuplicateInput(OutPoint),
    UtxoNotFound(OutPoint),
    InsufficientFunds {
        required: u64,
        available: u64,
    },
    AmountOverflow,
    OutputsExceedInputs,
    InvalidStateTransition {
        from: TransactionStatus,
        to: TransactionStatus,
    },
}

impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionError::EmptyInputs => write!(f, "transaction has no inputs"),
            TransactionError::EmptyOutputs => write!(f, "transaction has no outputs"),
            TransactionError::ZeroValueOutput => {
                write!(f, "output value must be greater than zero")
            }
            TransactionError::DuplicateInput(outpoint) => {
                write!(f, "input {outpoint} is spent more than once")
            }
            TransactionError::UtxoNotFound(outpoint) => write!(f, "no UTXO found for {outpoint}"),
            TransactionError::InsufficientFunds {
                required,
                available,
            } => write!(
                f,
                "insufficient funds: required {required} sats, only {available} sats available"
            ),
            TransactionError::AmountOverflow => write!(f, "amount overflowed a 64-bit integer"),
            TransactionError::OutputsExceedInputs => {
                write!(f, "output value exceeds input value: fee would be negative")
            }
            TransactionError::InvalidStateTransition { from, to } => {
                write!(f, "cannot transition transaction from {from} to {to}")
            }
        }
    }
}

impl std::error::Error for TransactionError {}
