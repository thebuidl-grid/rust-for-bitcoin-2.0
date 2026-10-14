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
        // TODO(Part 4): return a useful message for every error variant.
        // todo!("implement Display for TransactionError")

        match self {
            TransactionError::NoInputs => {
                write!(
                    f,
                    "transaction validation failed: transaction must contain at least one input"
                )
            }
            TransactionError::NoOutputs => {
                write!(
                    f,
                    "transaction validation failed: transaction must contain at least one output"
                )
            }
            TransactionError::ZeroValueOutput => {
                write!(
                    f,
                    "transaction validation failed: non-OpReturn output value cannot be zero"
                )
            }
            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => {
                write!(
                    f,
                    "transaction validation failed: total outputs ({total_outputs} sats) exceed total inputs ({total_inputs} sats)"
                )
            }
            TransactionError::CoinbaseMixedWithRegularInputs => {
                write!(
                    f,
                    "transaction validation failed: cannot mix coinbase and regular inputs"
                )
            }
            TransactionError::MultipleCoinbaseInputs => {
                write!(f, "transaction validation failed: transaction cannot contain more than one coinbase input")
            }
            TransactionError::InvalidTxid => {
                write!(f, "transaction validation failed: regular input contains an empty or invalid txid")
            }
            TransactionError::InsufficientFunds {
                available,
                required,
            } => {
                write!(
                    f,
                    "coin selection failed: insufficient funds available ({available} sats available, {required} sats required)"
                )
            }
        }
    }
}

impl std::error::Error for TransactionError {}
