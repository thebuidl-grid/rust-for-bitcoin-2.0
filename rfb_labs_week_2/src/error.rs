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
            TransactionError::NoInputs => write!(f, "transaction must contain at least one input"),
            TransactionError::NoOutputs => {
                write!(f, "transaction must contain at least one output")
            }
            TransactionError::ZeroValueOutput => {
                write!(f, "non-OP_RETURN output must have a non-zero value")
            }
            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => {
                write!(
                    f,
                    "outputs exceed inputs (total inputs: {total_inputs}, total outputs: {total_outputs})"
                )
            }
            TransactionError::CoinbaseMixedWithRegularInputs => {
                write!(f, "coinbase transaction cannot mix regular inputs")
            }
            TransactionError::MultipleCoinbaseInputs => {
                write!(f, "transaction cannot contain multiple coinbase inputs")
            }
            TransactionError::InvalidTxid => write!(f, "regular input has an empty txid"),
            TransactionError::InsufficientFunds {
                available,
                required,
            } => {
                write!(
                    f,
                    "insufficient funds (available: {available}, required: {required})"
                )
            }
        }
    }
}

impl std::error::Error for TransactionError {}
