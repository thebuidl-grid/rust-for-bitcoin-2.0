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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionError::NoInputs => write!(f, "Transaction must contain at least one input"),
            TransactionError::NoOutputs => {
                write!(f, "Transaction must contain at least one output")
            }
            TransactionError::ZeroValueOutput => {
                write!(
                    f,
                    "Transaction contains a non-OpReturn output with zero value"
                )
            }
            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => write!(
                f,
                "Outputs ({total_outputs} sats) exceed inputs ({total_inputs} sats)"
            ),
            TransactionError::CoinbaseMixedWithRegularInputs => {
                write!(
                    f,
                    "Transaction cannot mix coinbase inputs with regular inputs"
                )
            }
            TransactionError::MultipleCoinbaseInputs => {
                write!(f, "Transaction cannot contain multiple coinbase inputs")
            }
            TransactionError::InvalidTxid => {
                write!(f, "Regular input contains an invalid or empty TXID")
            }
            TransactionError::InsufficientFunds {
                available,
                required,
            } => write!(
                f,
                "Insufficient funds: available {available} sats, required {required} sats"
            ),
        }
    }
}

impl std::error::Error for TransactionError {}
