use thiserror::Error;

#[derive(Debug, Error)]
pub enum SerializeError {
    #[error("invalid hexadecimal value: {0}")]
    InvalidHex(#[from] hex::FromHexError),

    #[error("transaction must contain at least one input")]
    NoInputs,

    #[error("transaction must contain at least one output")]
    NoOutputs,

    #[error("input {input_index} has a {actual_length}-byte previous TXID; expected 32 bytes")]
    InvalidTxidLength {
        input_index: usize,
        actual_length: usize,
    },
}
