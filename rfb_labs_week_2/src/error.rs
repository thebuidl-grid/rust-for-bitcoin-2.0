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
}

impl fmt::Display for TransactionError {
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(Part 4): return a useful message for every error variant.
        match self {
            Self::NoInputs => write!(_formatter, "Transaction has no Inputs."),
            Self::NoOutputs => write!(_formatter, "Transaction has no Outputs"),
            Self::ZeroValueOutput => write!(_formatter, "Transaction output has zero value"),
            Self::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => write!(
                _formatter,
                "Total outputs ({}) exceed total inputs ({})",
                total_outputs, total_inputs
            ),
            Self::CoinbaseMixedWithRegularInputs => write!(
                _formatter,
                "Coinbase input cannot be mixed with regular inputs."
            ),
            Self::MultipleCoinbaseInputs => {
                write!(_formatter, "Transaction contains multiple coinbase inputs.")
            }
            Self::InvalidTxid => write!(_formatter, "Invalid transaction ID."),
            Self::InsufficientFunds {
                available,
                required,
            } => {
                write!(
                    _formatter,
                    "Insufficient funds: {} available, {} required",
                    available, required
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateTransitionError {
    InvalidTransition {
        from: TransactionState,
        to: TransactionState,
    },
    NotInRequiredState {
        current: TransactionState,
        required: TransactionState,
    },
}

impl fmt::Display for StateTransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransition { from, to } => {
                write!(f, "Cannot transition from {} to {}", from, to)
            }
            Self::NotInRequiredState { current, required } => {
                write!(
                    f,
                    "Transaction is in {} state, but {} state is required",
                    current, required
                )
            }
        }
    }
}

impl std::error::Error for StateTransitionError {}
impl std::error::Error for TransactionError {}
