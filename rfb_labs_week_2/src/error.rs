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
            Self::NoInputs => write!(formatter, "transaction must have at least one input"),
            Self::NoOutputs => write!(formatter, "transaction must have at least one output"),
            Self::ZeroValueOutput => write!(
                formatter,
                "transaction output value must be greater than zero unless it is OpReturn"
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
                "coinbase inputs cannot be mixed with regular inputs"
            ),
            Self::MultipleCoinbaseInputs => write!(
                formatter,
                "transaction cannot contain more than one coinbase input"
            ),
            Self::InvalidTxid => write!(formatter, "regular input txid cannot be empty"),
            Self::InsufficientFunds {
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
