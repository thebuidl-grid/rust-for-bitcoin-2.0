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
                "zero-value outputs are only allowed for OP_RETURN"
            ),
            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => write!(
                formatter,
                "total output value ({total_outputs} sats) exceeds total input value ({total_inputs} sats)"
            ),
            TransactionError::CoinbaseMixedWithRegularInputs => write!(
                formatter,
                "a coinbase input cannot be mixed with regular inputs"
            ),
            TransactionError::MultipleCoinbaseInputs => {
                write!(formatter, "a transaction cannot have more than one coinbase input")
            }
            TransactionError::InvalidTxid => {
                write!(formatter, "regular input has an empty or invalid txid")
            }
            TransactionError::InsufficientFunds {
                available,
                required,
            } => write!(
                formatter,
                "insufficient funds: available {available} sats, required {required} sats"
            ),
        }
    }
}

impl std::error::Error for TransactionError {}
