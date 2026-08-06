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
                write!(formatter, "transaction has no inputs")
            }

            TransactionError::NoOutputs => {
                write!(formatter, "transaction has no outputs")
            }

            TransactionError::ZeroValueOutput => {
                write!(formatter, "non-OP_RETURN outputs cannot have zero value")
            }

            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => {
                write!(
                    formatter,
                    "outputs exceed inputs: inputs={}, outputs={}",
                    total_inputs, total_outputs
                )
            }

            TransactionError::CoinbaseMixedWithRegularInputs => {
                write!(
                    formatter,
                    "coinbase inputs cannot be mixed with regular inputs"
                )
            }

            TransactionError::MultipleCoinbaseInputs => {
                write!(
                    formatter,
                    "transaction cannot contain multiple coinbase inputs"
                )
            }

            TransactionError::InvalidTxid => {
                write!(formatter, "regular input has an empty transaction id")
            }

            TransactionError::InsufficientFunds {
                available,
                required,
            } => {
                write!(
                    formatter,
                    "insufficient funds: available={}, required={}",
                    available, required
                )
            }
        }
    }
}

impl std::error::Error for TransactionError {}
