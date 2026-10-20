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
                write!(formatter, "transaction must have at least one input")
            }
            TransactionError::NoOutputs => {
                write!(formatter, "transaction must have at least one output")
            }
            TransactionError::ZeroValueOutput => write!(
                formatter,
                "non-op_return outputs must have a non-zero value"
            ),
            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => write!(
                formatter,
                "outputs ({total_outputs} sats) exceed inputs ({total_inputs} sats)"
            ),
            TransactionError::CoinbaseMixedWithRegularInputs => write!(
                formatter,
                "coinbase inputs cannot be mixed with regular inputs"
            ),
            TransactionError::MultipleCoinbaseInputs => write!(
                formatter,
                "transaction cannot contain more than one coinbase input"
            ),
            TransactionError::InvalidTxid => {
                write!(formatter, "regular input txid cannot be empty")
            }
            TransactionError::InsufficientFunds {
                available,
                required,
            } => write!(
                formatter,
                "insufficient funds: {available} sats available, {required} sats required"
            ),
        }
    }
}

impl std::error::Error for TransactionError {}
