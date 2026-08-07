use crate::state::TxState;
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
    InvalidStateTransition {
        from: TxState,
        to: TxState,
    },
}

impl fmt::Display for TransactionError {
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionError::NoInputs => write!(_formatter, "This Transaction has NO INPUT"),
            TransactionError::NoOutputs => write!(_formatter, "This Transaction has NO OUTPUT"),
            TransactionError::ZeroValueOutput => write!(_formatter, "Output has Zero Value and This is Not an OpReturn"),
            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            } => write!(_formatter,"Total Output ({total_outputs} sats) is greater than Total Input ({total_inputs} sats)"),
            TransactionError::CoinbaseMixedWithRegularInputs =>{
                write!(_formatter, "Coinbase Input cannot be mixed with Regular Input")
            }
            TransactionError::MultipleCoinbaseInputs =>{
                write!(_formatter, "A Transaction cannot contain more than one Coinbase Input")
            }
            TransactionError::InvalidTxid => write!(_formatter, "A Regular Input has an Invalid txid"),
            TransactionError::InsufficientFunds {
                available,
                required,
            } => write!(_formatter, "Insufficient Funds: Your available is {available} sats, The Required is {required} sats"),
            TransactionError::InvalidStateTransition { from, to } => write!(
                _formatter, "cannot transition transaction state from {from:?} to {to:?}"),

        }
    }
}

impl std::error::Error for TransactionError {}
