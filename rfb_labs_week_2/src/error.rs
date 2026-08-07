use std::fmt;
use crate::transaction::TransactionState;

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
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionError::NoInputs => write!(_formatter, "transaction has no inputs"),
            TransactionError::NoOutputs => write!(_formatter, "transaction has no outputs"),
            TransactionError::ZeroValueOutput => {
                write!(_formatter, "transaction has an output with zero value")
            }
            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => write!(
                _formatter,
                "transaction outputs ({}) exceed inputs ({})",
                total_outputs, total_inputs
            ),
            TransactionError::CoinbaseMixedWithRegularInputs => {
                write!(_formatter, "coinbase input mixed with regular inputs")
            }
            TransactionError::MultipleCoinbaseInputs => {
                write!(_formatter, "multiple coinbase inputs in transaction")
            }
            TransactionError::InvalidTxid => write!(_formatter, "A regular input has an empty txid"),
            TransactionError::InsufficientFunds { available, required } => write!(
                _formatter,
                "insufficient funds: available {}, required {}",
                available, required
            ),
            TransactionError::InvalidStateTransition { from, to } => {
                write!(_formatter, "cannot transition from {from:?} to {to:?}")
            }



        }
    }
}

impl std::error::Error for TransactionError {}
