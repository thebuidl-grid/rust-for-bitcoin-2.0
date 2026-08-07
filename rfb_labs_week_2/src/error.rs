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
            TransactionError::ZeroValueOutput => {
                write!(f, "Transaction contains a non-OP_RETURN zero-value output")
            }
            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => write!(
                f,
                "Outputs exceed inputs: total inputs = {} sats, total outputs = {} sats",
                total_inputs, total_outputs
            ),
            TransactionError::CoinbaseMixedWithRegularInputs => {
                write!(f, "Cannot mix coinbase inputs with regular inputs")
            }
            TransactionError::MultipleCoinbaseInputs => {
                write!(f, "Transaction has multiple coinbase inputs")
            }
            TransactionError::InvalidTxid => {
                write!(
                    f,
                    "Input has an invalid TXID (must be a 64-character hex string)"
                )
            }
            TransactionError::InsufficientFunds {
                available,
                required,
            } => write!(
                f,
                "Insufficient funds: available = {} sats, required = {} sats",
                available, required
            ),
        }
    }
}

impl std::error::Error for TransactionError {}
