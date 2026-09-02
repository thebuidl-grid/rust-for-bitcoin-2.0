//! Every way a user supplied value can be wrong.
//!
//! The command line is the only source of transaction data now, so the error
//! type carries enough context to point at the exact argument that is at
//! fault: which `--input`, which key inside it, which character of the hex.

use std::fmt;

/// A validation failure. Nothing here is a bug in the program, it is always
/// something the caller typed, so every variant knows how to describe itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxError {
    /// Two hex characters make one byte, so an odd count cannot be split.
    OddHexLength {
        field: String,
        len: usize,
    },
    /// `0x1234` is a number literal; Bitcoin byte strings are written bare.
    HexPrefix {
        field: String,
    },
    /// Something outside `0-9a-fA-F` turned up in a byte string.
    InvalidHexChar {
        field: String,
        character: char,
        position: usize,
    },
    /// A txid is always 32 bytes.
    TxidLength {
        field: String,
        len: usize,
    },
    /// Not parseable as an integer at all.
    NotANumber {
        field: String,
        value: String,
    },
    /// Parsed, but wider than the field it has to fit into.
    OutOfRange {
        field: String,
        value: String,
        limit: String,
    },
    /// The shape of a whole `--input` / `--output` / `--witness` value is wrong.
    MalformedSpec {
        field: String,
        value: String,
        expected: &'static str,
    },
    UnknownKey {
        field: String,
        key: String,
        valid: &'static str,
    },
    DuplicateKey {
        field: String,
        key: String,
    },
    MissingKey {
        field: String,
        key: &'static str,
    },
    /// `--witness 3:...` when there are only two inputs.
    WitnessIndex {
        index: usize,
        inputs: usize,
    },
    /// The same input was given a witness stack twice.
    WitnessConflict {
        index: usize,
    },
    /// Witness items were supplied but the legacy format was forced.
    WitnessWithoutSegwit {
        items: usize,
    },
    /// The SegWit format was forced but there is nothing to put in it.
    SegwitWithoutWitness,
}

impl TxError {
    /// A second line of output telling the caller what to do about it. Not
    /// every error needs one; a missing key already says what is missing.
    pub fn hint(&self) -> Option<String> {
        match self {
            TxError::OddHexLength { .. } => {
                Some("add or remove a character so the value has an even length".to_string())
            }
            TxError::HexPrefix { .. } => {
                Some("write the bytes on their own, for example 0014a632c1ff...".to_string())
            }
            TxError::InvalidHexChar { .. } => {
                Some("hexadecimal digits are 0-9, a-f and A-F".to_string())
            }
            TxError::TxidLength { .. } => Some(
                "a txid is 32 bytes, so 64 hexadecimal characters, in the order shown by a block \
                 explorer (see --txid-order)"
                    .to_string(),
            ),
            TxError::MalformedSpec { expected, .. } => Some(format!("expected {expected}")),
            TxError::WitnessIndex { inputs, .. } => Some(format!(
                "inputs are numbered from 0, so the highest usable index is {}",
                inputs.saturating_sub(1)
            )),
            TxError::WitnessConflict { index } => Some(format!(
                "input {index} already carries a witness= key; give the stack in one place only"
            )),
            TxError::WitnessWithoutSegwit { .. } => Some(
                "the legacy format has nowhere to put a witness: drop --no-segwit, or drop the \
                 witness items"
                    .to_string(),
            ),
            TxError::SegwitWithoutWitness => Some(
                "BIP144 keeps the legacy format for a transaction whose witness stacks are all \
                 empty: drop --segwit, or add witness items"
                    .to_string(),
            ),
            _ => None,
        }
    }
}

impl fmt::Display for TxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxError::OddHexLength { field, len } => write!(
                f,
                "{field}: hexadecimal value has an odd length ({len} characters)"
            ),
            TxError::HexPrefix { field } => {
                write!(f, "{field}: hexadecimal value must not start with `0x`")
            }
            TxError::InvalidHexChar {
                field,
                character,
                position,
            } => write!(
                f,
                "{field}: `{character}` is not a hexadecimal digit (position {position})"
            ),
            TxError::TxidLength { field, len } => {
                write!(f, "{field}: a txid must be 32 bytes, got {len}")
            }
            TxError::NotANumber { field, value } => {
                write!(f, "{field}: `{value}` is not a whole number")
            }
            TxError::OutOfRange {
                field,
                value,
                limit,
            } => write!(f, "{field}: {value} is out of range (limit {limit})"),
            TxError::MalformedSpec { field, value, .. } => {
                write!(f, "{field}: cannot read `{value}`")
            }
            TxError::UnknownKey { field, key, valid } => {
                write!(f, "{field}: unknown key `{key}`, expected one of {valid}")
            }
            TxError::DuplicateKey { field, key } => {
                write!(f, "{field}: `{key}` was given more than once")
            }
            TxError::MissingKey { field, key } => {
                write!(f, "{field}: missing required key `{key}`")
            }
            TxError::WitnessIndex { index, inputs } => write!(
                f,
                "--witness {index}: there is no input {index}, the transaction has {inputs}"
            ),
            TxError::WitnessConflict { index } => {
                write!(f, "--witness {index}: input {index} already has a witness")
            }
            TxError::WitnessWithoutSegwit { items } => write!(
                f,
                "--no-segwit was given, but {items} witness item(s) were supplied"
            ),
            TxError::SegwitWithoutWitness => {
                write!(f, "--segwit was given, but no witness items were supplied")
            }
        }
    }
}

impl std::error::Error for TxError {}

pub type Result<T> = std::result::Result<T, TxError>;
