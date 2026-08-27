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
            TransactionError::NoInputs => write!(f, "Transaction has no inputs"),
            TransactionError::NoOutputs => write!(f, "Transaction has no outputs"),
            TransactionError::ZeroValueOutput => write!(
                f,
                "Transaction output value cannot be zero (unless it is OpReturn)"
            ),
            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => {
                write!(
                    f,
                    "Outputs ({}) exceed inputs ({})",
                    total_outputs, total_inputs
                )
            }
            TransactionError::CoinbaseMixedWithRegularInputs => {
                write!(f, "Coinbase inputs cannot be mixed with regular inputs")
            }
            TransactionError::MultipleCoinbaseInputs => {
                write!(f, "Transaction cannot have multiple coinbase inputs")
            }
            TransactionError::InvalidTxid => {
                write!(f, "Transaction input previous TXID is invalid (empty)")
            }
            TransactionError::InsufficientFunds {
                available,
                required,
            } => {
                write!(
                    f,
                    "Insufficient funds: available {}, required {}",
                    available, required
                )
            }
        }
    }
}

impl std::error::Error for TransactionError {}
