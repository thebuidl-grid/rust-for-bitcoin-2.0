use thiserror::Error;

/// Errors that can occur while decoding a raw Bitcoin transaction.
#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("invalid transaction hex: {0}")]
    InvalidHex(#[from] hex::FromHexError),

    #[error("transaction data is incomplete: expected more bytes")]
    Io(#[from] std::io::Error),

    #[error("invalid CompactSize encoding: value is not encoded in its shortest form")]
    InvalidCompactSize,

    #[error("script length {0} cannot be represented on this platform")]
    ScriptLengthTooLarge(u64),

    #[error("invalid SegWit marker and flag: expected 00 01")]
    InvalidSegwitMarker,

    #[error("transaction contains {0} unparsed trailing byte(s)")]
    TrailingBytes(usize),

    #[error("failed to serialize the decoded transaction: {0}")]
    Serialization(#[from] serde_json::Error),
}
