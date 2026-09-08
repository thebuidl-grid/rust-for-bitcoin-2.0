use std::fmt;

/// Errors produced while decoding a hex string into bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HexError {
    OddLength,
    InvalidChar { ch: char, index: usize },
}

impl fmt::Display for HexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HexError::OddLength => write!(f, "hex string has an odd number of characters"),
            HexError::InvalidChar { ch, index } => {
                write!(f, "invalid hex character '{ch}' at position {index}")
            }
        }
    }
}

impl std::error::Error for HexError {}

/// Decode a hex string into bytes, validating every character before conversion.
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, HexError> {
    if !hex.len().is_multiple_of(2) {
        return Err(HexError::OddLength);
    }
    if let Some((index, ch)) = hex.char_indices().find(|(_, c)| !c.is_ascii_hexdigit()) {
        return Err(HexError::InvalidChar { ch, index });
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16).expect("chars validated as hex above");
        bytes.push(byte);
    }
    Ok(bytes)
}

/// Errors produced while parsing a `--input` / `--output` argument value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxArgError {
    InvalidPair {
        arg: &'static str,
        raw: String,
    },
    UnknownField {
        arg: &'static str,
        field: String,
    },
    MissingField {
        arg: &'static str,
        field: &'static str,
    },
    InvalidHex {
        arg: &'static str,
        field: &'static str,
        raw: String,
        source: HexError,
    },
    InvalidInt {
        arg: &'static str,
        field: &'static str,
        raw: String,
        reason: String,
    },
    InvalidTxidLength {
        length: usize,
    },
}

impl fmt::Display for TxArgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxArgError::InvalidPair { arg, raw } => write!(
                f,
                "{arg}: invalid entry '{raw}', expected 'key=value' separated by commas"
            ),
            TxArgError::UnknownField { arg, field } => {
                write!(f, "{arg}: unknown field '{field}'")
            }
            TxArgError::MissingField { arg, field } => {
                write!(f, "{arg}: missing required field '{field}'")
            }
            TxArgError::InvalidHex {
                arg,
                field,
                raw,
                source,
            } => write!(
                f,
                "{arg}: field '{field}' has invalid hex value '{raw}': {source}"
            ),
            TxArgError::InvalidInt {
                arg,
                field,
                raw,
                reason,
            } => write!(
                f,
                "{arg}: field '{field}' has invalid integer value '{raw}': {reason}"
            ),
            TxArgError::InvalidTxidLength { length } => write!(
                f,
                "--input: field 'txid' must decode to exactly 32 bytes, got {length}"
            ),
        }
    }
}

impl std::error::Error for TxArgError {}
