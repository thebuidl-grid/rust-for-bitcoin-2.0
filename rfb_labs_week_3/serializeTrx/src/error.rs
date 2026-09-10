use thiserror::Error;

#[derive(Debug, Error)]
pub enum TxSerializerError {
    #[error("Invalid hexadecimal string: {0}")]
    InvalidHex(String),

    #[error("Hexadecimal string must have even length, got {0}")]
    OddHexLength(usize),

    #[error("Invalid transaction version: {0}")]
    InvalidVersion(String),

    #[error("Invalid vout value: {0}")]
    InvalidVout(String),

    #[error("Invalid sequence number: {0}")]
    InvalidSequence(String),

    #[error("Invalid satoshi value: {0}")]
    InvalidValue(String),

    #[error("Invalid locktime: {0}")]
    InvalidLocktime(String),

    #[error("No inputs provided")]
    NoInputs,

    #[error("No outputs provided")]
    NoOutputs,

    #[error("Input {index} has no witness data but SegWit is enabled")]
    MissingWitness { index: usize },

    #[error("Previous transaction ID must be 32 bytes (64 hex characters), got {0}")]
    InvalidTxIdLength(usize),

    #[error("Invalid output format: {0}. Expected 'value:script_hex'")]
    InvalidOutputFormat(String),

    #[error(
        "Invalid input format: {0}. Expected 'prev_txid:vout' or 'prev_txid:vout:script_sig:sequence'"
    )]
    InvalidInputFormat(String),

    #[error("Argument parsing error: {0}")]
    ArgError(String),

    #[error("Invalid witness count format: {0}. Expected comma-separated integers")]
    InvalidWitnessCountFormat(String),
}

impl From<std::num::ParseIntError> for TxSerializerError {
    fn from(err: std::num::ParseIntError) -> Self {
        TxSerializerError::InvalidValue(err.to_string())
    }
}

impl From<hex::FromHexError> for TxSerializerError {
    fn from(err: hex::FromHexError) -> Self {
        TxSerializerError::InvalidHex(err.to_string())
    }
}
