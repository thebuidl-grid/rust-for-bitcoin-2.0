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
        // todo!("implement Display for TransactionError")
        match self {
            TransactionError::NoInputs => {
                write!(formatter, "transaction has no inputs")
            }
            TransactionError::NoOutputs => {
                write!(formatter, "transaction has no outputs")
            }
            TransactionError::ZeroValueOutput => {
                write!(formatter, "transaction contains an output with zero value")
            }
            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => write!(
                formatter,
                "outputs exceed inputs: total inputs {} sats, total outputs {} sats",
                total_inputs, total_outputs
            ),
            TransactionError::CoinbaseMixedWithRegularInputs => {
                write!(formatter, "coinbase input cannot be mixed with regular inputs")
            }
            TransactionError::MultipleCoinbaseInputs => {
                write!(formatter, "transaction contains more than one coinbase input")
            }
            TransactionError::InvalidTxid => {
                write!(formatter, "previous output references an invalid txid")
            }
            TransactionError::InsufficientFunds {
                available,
                required,
            } => write!(
                formatter,
                "insufficient funds: available {} sats, required {} sats",
                available, required
            ),
        }
    }
}

impl std::error::Error for TransactionError {}
