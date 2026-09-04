//! Error type for the parser. Every failure says which byte offset tripped it,
//! because "the transaction is invalid" is not a useful thing to tell a caller.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Input hex string was not valid hex.
    BadHex(String),
    /// Ran off the end of the buffer while reading `what`.
    UnexpectedEof { offset: usize, what: &'static str },
    /// A CompactSize integer used a longer encoding than its value needs.
    NonMinimalCompactSize { offset: usize },
    /// SegWit marker byte present but the flag byte was not `0x01`.
    BadSegwitFlag { offset: usize, flag: u8 },
    /// A SegWit transaction claimed zero inputs (the marker/flag would be ambiguous).
    EmptySegwitInputs,
    /// Bytes were left over after a full transaction was parsed.
    TrailingBytes { consumed: usize, total: usize },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadHex(msg) => write!(f, "input is not valid hex: {msg}"),
            Self::UnexpectedEof { offset, what } => {
                write!(
                    f,
                    "unexpected end of data at byte {offset} while reading {what}"
                )
            }
            Self::NonMinimalCompactSize { offset } => {
                write!(f, "non-minimal CompactSize encoding at byte {offset}")
            }
            Self::BadSegwitFlag { offset, flag } => {
                write!(f, "invalid SegWit flag byte {flag:#04x} at byte {offset}")
            }
            Self::EmptySegwitInputs => {
                write!(f, "SegWit transaction declares zero inputs")
            }
            Self::TrailingBytes { consumed, total } => {
                write!(
                    f,
                    "{} trailing byte(s) after transaction ({consumed} of {total} consumed)",
                    total - consumed
                )
            }
        }
    }
}

impl std::error::Error for ParseError {}

pub type Result<T> = std::result::Result<T, ParseError>;
