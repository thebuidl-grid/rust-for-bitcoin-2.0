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

        match self {
            TransactionError::NoInputs => {
                write!(formatter, "NoInputs: Transaction has no Inputs")
            }
            TransactionError::NoOutputs => {
                write!(formatter, "NoOutputs: Transaction has no Outputs")
            }
            TransactionError::ZeroValueOutput => {
                write!(
                    formatter,
                    "ZeroValueOutput: Transaction Output ia Zero and Not OP_RETURN"
                )
            }
            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => {
                write!(formatter, "OutputsExceedInputs: Transaction outputs: {total_outputs} exceeds Transaction Inputs: {total_inputs}")
            }
            TransactionError::CoinbaseMixedWithRegularInputs => {
                write!(formatter, "CoinbaseMixedWithRegularInputs: Transaction contains Both Coinbase and Regu;ar Transaction Inputs")
            }
            TransactionError::MultipleCoinbaseInputs => {
                write!(formatter, "MultipleCoinbaseInputs: Transaction contains more than one Coinbase Transaction Input")
            }
            TransactionError::InvalidTxid => {
                write!(
                    formatter,
                    "InvalidTxid: Transaction contains an invalid TXID"
                )
            }
            TransactionError::InsufficientFunds {
                available,
                required,
            } => {
                write!(
                    formatter,
                    "InsufficientFunds: Transaction need {required} sats but ave {available} sats"
                )
            }
        }
    }
}

impl std::error::Error for TransactionError {}
