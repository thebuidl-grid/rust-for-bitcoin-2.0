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
        // TODO(Part 4): return a useful message for every error variant.
        match self {
            TransactionError::NoInputs => {
                write!(formatter, "transaction has no inputs")
            }
            TransactionError::NoOutputs => {
                write!(formatter, "transaction has no outputs")
            }
            TransactionError::ZeroValueOutput => {
                write!(
                    formatter,
                    "transaction contains a zero-value output that is not OP_RETURN"
                )
            }
            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => {
                write!(
                    formatter,
                    "total outputs ({total_outputs} sats) exceed total inputs ({total_inputs} sats)"
                )
            }
            TransactionError::CoinbaseMixedWithRegularInputs => {
                write!(
                    formatter,
                    "coinbase input cannot be mixed with regular inputs"
                )
            }
            TransactionError::MultipleCoinbaseInputs => {
                write!(
                    formatter,
                    "transaction contains more than one coinbase input"
                )
            }
            TransactionError::InvalidTxid => {
                write!(formatter, "a regular input has an empty or invalid txid")
            }
            TransactionError::InsufficientFunds {
                available,
                required,
            } => {
                write!(
                    formatter,
                    "insufficient funds: have {available} sats, need {required} sats"
                )
            }
        }
    }
}

impl std::error::Error for TransactionError {}
