use std::fmt;

use crate::state::TransactionState;

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
        from: TransactionState,
        to: TransactionState,
    },
}

impl fmt::Display for TransactionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionError::NoInputs=> write!(formatter, "Input Error: No inputs found"),
            TransactionError::NoOutputs=> write!(formatter, "Output Error: No outputs found"),
            TransactionError::ZeroValueOutput=> write!(formatter, "The output value is zero"),
            TransactionError::OutputsExceedInputs { total_inputs, total_outputs } =>
             write!(formatter, "The total value outputs: ({total_outputs} sats) exceed the value of inputs: ({total_inputs} sats)"),
            TransactionError::CoinbaseMixedWithRegularInputs=> write!(formatter,"Coinbase is mixed with regular inputs"),
            TransactionError::MultipleCoinbaseInputs => write!(formatter, "Multiple Coinbase Inputs Found"),
            TransactionError::InvalidTxid => write!(formatter, "Invalid transaction id"),
            TransactionError::InsufficientFunds { available, required } =>
            write!(formatter, "Insufficient Funds, Available: {available} sats, Required: {required} sats"),
            TransactionError::InvalidStateTransition { from, to } =>
            write!(formatter, "Invalid State Transition from: {from} to: {to}"),
        }
    }
}

impl std::error::Error for TransactionError {}
