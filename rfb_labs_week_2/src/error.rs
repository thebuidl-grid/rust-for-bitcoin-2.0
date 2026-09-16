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
        //   todo!("implement Display for TransactionError")
        
        match self {
            Self::NoInputs => write!(formatter, "Transaction must have at least one input"),
            Self::NoOutputs => write!(formatter, "Transaction must have at least one output"),
            Self::ZeroValueOutput => {
                write!(formatter, "Non-OP_RETURN outputs must have positive value")
            }
            Self::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => {
                write!(
                    formatter,
                    "Total outputs ({}) exceed total inputs ({})",
                    total_outputs, total_inputs
                )
            }
            Self::CoinbaseMixedWithRegularInputs => {
                write!(
                    formatter,
                    "Coinbase inputs cannot be mixed with regular inputs"
                )
            }
            Self::MultipleCoinbaseInputs => {
                write!(formatter, "Transaction can have at most one coinbase input")
            }
            Self::InvalidTxid => write!(formatter, "Regular input has empty TXID"),
            Self::InsufficientFunds {
                available,
                required,
            } => {
                write!(
                    formatter,
                    "Insufficient funds: available {} but {} required",
                    available, required
                )
            }
        }
    }
}

impl std::error::Error for TransactionError {}
