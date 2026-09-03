use std::fmt;

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
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionError::NoInputs => {
                write!(formatter, "Transaction must have at least one input")
            }
            TransactionError::NoOutputs => {
                write!(formatter, "Transaction must have at least one output")
            }
            TransactionError::ZeroValueOutput => {
                write!(formatter, "Non-OpReturn outputs cannot have zero value")
            }
            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => {
                write!(
                    formatter,
                    "Total outputs ({} sats) exceed total inputs ({} sats)",
                    total_outputs, total_inputs
                )
            }
            TransactionError::CoinbaseMixedWithRegularInputs => {
                write!(
                    formatter,
                    "Transaction cannot mix coinbase and regular inputs"
                )
            }
            TransactionError::MultipleCoinbaseInputs => {
                write!(
                    formatter,
                    "Transaction cannot have multiple coinbase inputs"
                )
            }
            TransactionError::InvalidTxid => {
                write!(
                    formatter,
                    "Regular input cannot have an empty transaction ID"
                )
            }
            TransactionError::InsufficientFunds {
                available,
                required,
            } => {
                write!(
                    formatter,
                    "Insufficient funds: available {} sats, required {} sats",
                    available, required
                )
            }
        }
    }
}

impl std::error::Error for TransactionError {}
