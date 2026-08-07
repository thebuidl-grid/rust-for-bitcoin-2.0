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
            TransactionError::NoInputs => write!(formatter, "transaction has no inputs"),
            TransactionError::NoOutputs => write!(formatter, "transaction has no outputs"),
            TransactionError::ZeroValueOutput => write!(
                formatter,
                "transaction has a zero-value output that is not OpReturn"
            ),
            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => {
                write!(
                    formatter,
                    "outputs ({} sats) exceed inputs ({} sats)",
                    total_outputs, total_inputs
                )
            }
            TransactionError::CoinbaseMixedWithRegularInputs => {
                write!(formatter, "transaction mixes coinbase and regular inputs")
            }
            TransactionError::MultipleCoinbaseInputs => {
                write!(formatter, "transaction has multiple coinbase inputs")
            }
            TransactionError::InvalidTxid => write!(formatter, "transaction has an empty txid"),
            TransactionError::InsufficientFunds {
                available,
                required,
            } => {
                write!(
                    formatter,
                    "insufficient funds: available {} sats, required {} sats",
                    available, required
                )
            }
        }
    }
}

impl std::error::Error for TransactionError {}
