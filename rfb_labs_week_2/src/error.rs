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
            TransactionError::ZeroValueOutput => {
                write!(formatter, "non-OP_RETURN outputs must have a value")
            }
            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => write!(
                formatter,
                "output total {total_outputs} sats exceeds input total {total_inputs} sats"
            ),
            TransactionError::CoinbaseMixedWithRegularInputs => write!(
                formatter,
                "coinbase inputs cannot be mixed with regular inputs"
            ),
            TransactionError::MultipleCoinbaseInputs => write!(
                formatter,
                "a transaction cannot have multiple coinbase inputs"
            ),

            TransactionError::InvalidTxid => write!(
                formatter,
                "regular transaction input has an empty transaction ID"
            ),
            TransactionError::InsufficientFunds {
                available,
                required,
            } => write!(
                formatter,
                "insufficient funds: {required} sats required, {available} sats available"
            ),
        }
    }
}

impl std::error::Error for TransactionError {}
