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
            Self::NoInputs => write!(formatter, "transaction has no inputs"),
            Self::NoOutputs => write!(formatter, "transaction has no outputs"),
            Self::ZeroValueOutput => write!(
                formatter,
                "output has zero value but is not an OP_RETURN output"
            ),
            Self::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => write!(
                formatter,
                "total output value ({total_outputs} sats) exceeds total input value ({total_inputs} sats)"
            ),
            Self::CoinbaseMixedWithRegularInputs => write!(
                formatter,
                "a coinbase input cannot be mixed with regular inputs"
            ),
            Self::MultipleCoinbaseInputs => write!(
                formatter,
                "a transaction cannot have more than one coinbase input"
            ),
            Self::InvalidTxid => write!(formatter, "regular input has an empty txid"),
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
