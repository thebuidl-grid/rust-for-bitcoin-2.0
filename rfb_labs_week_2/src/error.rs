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
            Self::NoInputs => {
                write!(formatter, "transaction has no inputs: nothing is being spent")
            }
            Self::NoOutputs => write!(
                formatter,
                "transaction has no outputs: the entire input value would be paid as fee"
            ),
            Self::ZeroValueOutput => write!(
                formatter,
                "zero-value output is only allowed for OP_RETURN outputs"
            ),
            Self::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => write!(
                formatter,
                "outputs exceed inputs: {total_outputs} sats spent against {total_inputs} sats available"
            ),
            Self::CoinbaseMixedWithRegularInputs => write!(
                formatter,
                "a coinbase input cannot be combined with regular inputs"
            ),
            Self::MultipleCoinbaseInputs => write!(
                formatter,
                "a coinbase transaction must have exactly one coinbase input"
            ),
            Self::InvalidTxid => write!(
                formatter,
                "regular input refers to an empty txid: the previous output cannot be identified"
            ),
            Self::InsufficientFunds {
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
