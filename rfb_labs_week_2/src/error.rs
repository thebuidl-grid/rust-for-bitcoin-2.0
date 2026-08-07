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
            TransactionError::NoInputs => formatter.write_str("transaction has no inputs"),
            TransactionError::NoOutputs => formatter.write_str("transaction has no outputs"),
            TransactionError::ZeroValueOutput => {
                formatter.write_str("a non-OP_RETURN output has a zero value")
            }
            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => write!(
                formatter,
                "outputs ({total_outputs} sats) exceed inputs ({total_inputs} sats)"
            ),
            TransactionError::CoinbaseMixedWithRegularInputs => {
                formatter.write_str("a coinbase input is mixed with regular inputs")
            }
            TransactionError::MultipleCoinbaseInputs => {
                formatter.write_str("transaction has more than one coinbase input")
            }
            TransactionError::InvalidTxid => {
                formatter.write_str("a regular input has an empty transaction ID")
            }
            TransactionError::InsufficientFunds {
                available,
                required,
            } => {
                write!(
                    formatter,
                    "insufficient funds: {available} sats available, {required} sats required"
                )
            }
        }
    }
}

impl std::error::Error for TransactionError {}
