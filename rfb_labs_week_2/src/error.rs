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
                write!(formatter, "a transaction must spend at least one input")
            }
            TransactionError::NoOutputs => {
                write!(formatter, "a transaction must create at least one output")
            }
            TransactionError::ZeroValueOutput => write!(
                formatter,
                "only an OP_RETURN output may carry a value of zero satoshis"
            ),
            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => write!(
                formatter,
                "outputs spend {total_outputs} sats but the inputs only provide {total_inputs} sats"
            ),
            TransactionError::CoinbaseMixedWithRegularInputs => write!(
                formatter,
                "a coinbase input cannot be combined with regular inputs"
            ),
            TransactionError::MultipleCoinbaseInputs => write!(
                formatter,
                "a coinbase transaction must have exactly one coinbase input"
            ),
            TransactionError::InvalidTxid => {
                write!(formatter, "a regular input must reference a non-empty txid")
            }
            TransactionError::InsufficientFunds {
                available,
                required,
            } => write!(
                formatter,
                "insufficient funds: {available} sats available but {required} sats required"
            ),
        }
    }
}

impl std::error::Error for TransactionError {}
