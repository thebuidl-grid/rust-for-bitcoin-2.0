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
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(Part 4): return a useful message for every error variant.
        match self {
            TransactionError::NoInputs => write!(_formatter, "Transaction contains no inputs"),
            TransactionError::NoOutputs => write!(_formatter, "Transaction contains no outputs"),
            TransactionError::ZeroValueOutput => {
                write!(_formatter, "Non-OpReturn output has zero value")
            }
            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => {
                write!(
                    _formatter,
                    "Outputs ({total_outputs} sats) exceed inputs ({total_inputs} sats)"
                )
            }
            TransactionError::CoinbaseMixedWithRegularInputs => {
                write!(
                    _formatter,
                    "Cannot mix coinbase and regular inputs in the same transaction"
                )
            }
            TransactionError::MultipleCoinbaseInputs => {
                write!(_formatter, "Transaction contains multiple coinbase inputs")
            }
            TransactionError::InvalidTxid => {
                write!(_formatter, "Regular input contains an empty TXID")
            }
            TransactionError::InsufficientFunds {
                available,
                required,
            } => {
                write!(
                    _formatter,
                    "Insufficient funds: available {available} sats, required {required} sats"
                )
            }
        }
    }
}

impl std::error::Error for TransactionError {}
