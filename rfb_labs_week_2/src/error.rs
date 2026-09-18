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
            TransactionError::NoInputs => write!(f, "transaction has no inputs"),
            TransactionError::NoOutputs => write!(f, "..."),
            TransactionError::ZeroValueOutput => write!(f, "..."),
            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => {
                write!(
                    f,
                    "outputs ({total_outputs}) exceed inputs ({total_inputs})"
                )
            }
            TransactionError::CoinbaseMixedWithRegularInputs => write!(f, "..."),
            TransactionError::MultipleCoinbaseInputs => write!(f, "..."),
            TransactionError::InvalidTxid => write!(f, "..."),
            TransactionError::InsufficientFunds {
                available,
                required,
            } => {
                write!(f, "insufficient funds: have {available}, need {required}")
            }
        }
    }
}

impl std::error::Error for TransactionError {}
