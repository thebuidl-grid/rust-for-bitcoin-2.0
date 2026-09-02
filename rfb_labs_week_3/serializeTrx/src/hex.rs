//! Hex to bytes, with the checking done up front.
//!
//! Every byte string in a transaction (txid, scriptSig, scriptPubKey, witness
//! item) reaches the program as hex typed by a person, so this is the layer
//! where a typo has to be caught. It is the same conversion the original
//! program did, with the failure cases named instead of unwrapped.

use crate::error::{Result, TxError};

/// Decode `value` into bytes, blaming `field` if it is not valid hexadecimal.
///
/// `field` is the argument the value came from, e.g. `--input #2 txid`, so the
/// message can point straight at it.
pub fn hex_to_bytes(field: &str, value: &str) -> Result<Vec<u8>> {
    if value.starts_with("0x") || value.starts_with("0X") {
        return Err(TxError::HexPrefix {
            field: field.to_string(),
        });
    }

    // Character check first: it also guarantees the string is pure ASCII, so
    // the byte offsets used to slice it below always land on a boundary.
    for (position, character) in value.char_indices() {
        if !character.is_ascii_hexdigit() {
            return Err(TxError::InvalidHexChar {
                field: field.to_string(),
                character,
                position,
            });
        }
    }

    if value.len() % 2 != 0 {
        return Err(TxError::OddHexLength {
            field: field.to_string(),
            len: value.len(),
        });
    }

    let mut bytes = Vec::with_capacity(value.len() / 2);
    for i in (0..value.len()).step_by(2) {
        // Two hex characters make one byte. Validated above, so this cannot fail.
        bytes.push(u8::from_str_radix(&value[i..i + 2], 16).expect("validated hex digits"));
    }

    Ok(bytes)
}

/// Decode a 32 byte hash, rejecting anything of the wrong width.
pub fn hex_to_txid(field: &str, value: &str) -> Result<Vec<u8>> {
    let bytes = hex_to_bytes(field, value)?;
    if bytes.len() != 32 {
        return Err(TxError::TxidLength {
            field: field.to_string(),
            len: bytes.len(),
        });
    }
    Ok(bytes)
}

/// The other direction, used to print the finished transaction.
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_and_re_encodes() {
        assert_eq!(hex_to_bytes("f", "00ff10").unwrap(), vec![0x00, 0xff, 0x10]);
        assert_eq!(hex_to_bytes("f", "").unwrap(), Vec::<u8>::new());
        // Case does not matter on the way in; output is always lowercase.
        assert_eq!(bytes_to_hex(&hex_to_bytes("f", "AbCd").unwrap()), "abcd");
    }

    #[test]
    fn rejects_bad_hex_before_converting() {
        assert_eq!(
            hex_to_bytes("f", "00f"),
            Err(TxError::OddHexLength {
                field: "f".to_string(),
                len: 3
            })
        );
        assert_eq!(
            hex_to_bytes("f", "00g0"),
            Err(TxError::InvalidHexChar {
                field: "f".to_string(),
                character: 'g',
                position: 2
            })
        );
        assert!(matches!(
            hex_to_bytes("f", "0x00ff"),
            Err(TxError::HexPrefix { .. })
        ));
        // A multi byte character is reported as a character, not as a length
        // problem, and never reaches the slicing below.
        assert!(matches!(
            hex_to_bytes("f", "00é"),
            Err(TxError::InvalidHexChar {
                character: 'é', ..
            })
        ));
    }

    #[test]
    fn a_txid_must_be_32_bytes() {
        let short = "00".repeat(31);
        assert_eq!(
            hex_to_txid("f", &short),
            Err(TxError::TxidLength {
                field: "f".to_string(),
                len: 31
            })
        );
        assert!(hex_to_txid("f", &"00".repeat(32)).is_ok());
    }
}
