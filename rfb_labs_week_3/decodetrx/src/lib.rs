use sha2::{Digest, Sha256};
use std::io::{Error, ErrorKind};
use transaction::{Amount, Input, Output, Transaction, Txid};

mod transaction;

pub(crate) fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn decode_hex(hex_str: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if !hex_str.len().is_multiple_of(2) {
        return Err("hex string has an odd number of characters".into());
    }
    let mut bytes = Vec::with_capacity(hex_str.len() / 2);
    for i in (0..hex_str.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex_str[i..i + 2], 16)?;
        bytes.push(byte);
    }
    Ok(bytes)
}

#[allow(dead_code)]
fn read_version(transaction_hex: &str) -> u32 {
    let bytes = decode_hex(&transaction_hex[0..8]).unwrap_or_default();
    let mut arr = [0u8; 4];
    arr.copy_from_slice(&bytes[..4]);
    u32::from_le_bytes(arr)
}

fn read_u64(transaction_bytes: &mut &[u8]) -> u64 {
    let bytes: [u8; 8] = transaction_bytes[..8]
        .try_into()
        .expect("not enough bytes remaining for a u64");
    *transaction_bytes = &transaction_bytes[8..];
    u64::from_le_bytes(bytes)
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    if transaction_bytes.len() < 8 {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "not enough bytes for amount",
        ));
    }
    Ok(Amount::from_sat(read_u64(transaction_bytes)))
}

fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    if bytes_slice.len() < 4 {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "not enough bytes for u32",
        ));
    }
    let bytes: [u8; 4] = bytes_slice[..4].try_into().unwrap();
    *bytes_slice = &bytes_slice[4..];
    Ok(u32::from_le_bytes(bytes))
}

fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    if transaction_bytes.is_empty() {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "empty bytes for compact size",
        ));
    }
    let first = transaction_bytes[0];
    *transaction_bytes = &transaction_bytes[1..];

    match first {
        0xfd => {
            if transaction_bytes.len() < 2 {
                return Err(Error::new(ErrorKind::UnexpectedEof, "not enough bytes"));
            }
            let bytes: [u8; 2] = transaction_bytes[..2].try_into().unwrap();
            *transaction_bytes = &transaction_bytes[2..];
            Ok(u16::from_le_bytes(bytes) as u64)
        }
        0xfe => {
            if transaction_bytes.len() < 4 {
                return Err(Error::new(ErrorKind::UnexpectedEof, "not enough bytes"));
            }
            let bytes: [u8; 4] = transaction_bytes[..4].try_into().unwrap();
            *transaction_bytes = &transaction_bytes[4..];
            Ok(u32::from_le_bytes(bytes) as u64)
        }
        0xff => {
            if transaction_bytes.len() < 8 {
                return Err(Error::new(ErrorKind::UnexpectedEof, "not enough bytes"));
            }
            let bytes: [u8; 8] = transaction_bytes[..8].try_into().unwrap();
            *transaction_bytes = &transaction_bytes[8..];
            Ok(u64::from_le_bytes(bytes))
        }
        n => Ok(n as u64),
    }
}

fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    if transaction_bytes.len() < 32 {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "not enough bytes for txid",
        ));
    }
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&transaction_bytes[..32]);
    *transaction_bytes = &transaction_bytes[32..];
    Ok(Txid::from_bytes(bytes))
}

#[allow(dead_code)]
fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    let script_len = read_compact_size(transaction_bytes)? as usize;
    if transaction_bytes.len() < script_len {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "not enough bytes for script",
        ));
    }
    let script_bytes = &transaction_bytes[..script_len];
    let hex_script = encode_hex(script_bytes);
    *transaction_bytes = &transaction_bytes[script_len..];
    Ok(hex_script)
}

#[allow(dead_code)]
fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    read_u32(transaction_bytes)
}

fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    let first_hash = Sha256::digest(row_transaction_bytes);
    let second_hash = Sha256::digest(first_hash);
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&second_hash);
    Ok(Txid::from_bytes(bytes))
}

pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let raw_bytes = decode_hex(&transaction_hex)?;
    let mut bytes: &[u8] = &raw_bytes;

    let mut legacy_bytes: Vec<u8> = Vec::new();

    let before = bytes;
    let version = read_u32(&mut bytes)?;
    legacy_bytes.extend_from_slice(&before[..4]);

    if bytes.len() < 2 || bytes[0] != 0x00 || bytes[1] != 0x01 {
        return Err("expected SegWit marker (0x00) and flag (0x01)".into());
    }
    bytes = &bytes[2..];

    let before = bytes;
    let input_count = read_compact_size(&mut bytes)?;
    legacy_bytes.extend_from_slice(&before[..before.len() - bytes.len()]);

    let mut inputs = Vec::with_capacity(input_count as usize);
    for _ in 0..input_count {
        let input_start = bytes;

        let txid = read_txid(&mut bytes)?;
        let output_index = read_u32(&mut bytes)?;

        let script_len = read_compact_size(&mut bytes)? as usize;
        if bytes.len() < script_len {
            return Err(
                Error::new(ErrorKind::UnexpectedEof, "not enough bytes for script_sig").into(),
            );
        }
        let script_sig = bytes[..script_len].to_vec();
        bytes = &bytes[script_len..];

        let sequence = read_u32(&mut bytes)?;

        legacy_bytes.extend_from_slice(&input_start[..input_start.len() - bytes.len()]);
        inputs.push(Input {
            txid,
            output_index,
            script_sig,
            sequence,
        });
    }

    let before = bytes;
    let output_count = read_compact_size(&mut bytes)?;
    legacy_bytes.extend_from_slice(&before[..before.len() - bytes.len()]);

    let mut outputs = Vec::with_capacity(output_count as usize);
    for _ in 0..output_count {
        let output_start = bytes;

        let amount = read_amount(&mut bytes)?;

        let script_len = read_compact_size(&mut bytes)? as usize;
        if bytes.len() < script_len {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "not enough bytes for script_pubkey",
            )
            .into());
        }
        let script_pubkey = bytes[..script_len].to_vec();
        bytes = &bytes[script_len..];

        legacy_bytes.extend_from_slice(&output_start[..output_start.len() - bytes.len()]);
        outputs.push(Output {
            amount,
            script_pubkey,
        });
    }

    for _ in 0..input_count {
        let witness_count = read_compact_size(&mut bytes)?;
        for _ in 0..witness_count {
            let item_len = read_compact_size(&mut bytes)? as usize;
            if bytes.len() < item_len {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "not enough bytes for witness item",
                )
                .into());
            }
            bytes = &bytes[item_len..];
        }
    }

    let before = bytes;
    let lock_time = read_u32(&mut bytes)?;
    legacy_bytes.extend_from_slice(&before[..4]);

    let transaction_id = hash_row_transaction(&legacy_bytes)?;

    let transaction = Transaction {
        transaction_id,
        version,
        inputs,
        outputs,
        lock_time,
    };

    Ok(serde_json::to_string_pretty(&transaction)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact transaction you already manually verified against the
    /// reference `trxparse` script and cross-checked field by field.
    const KNOWN_TX_HEX: &str = "0200000000010196277c04c986c1ad78c909287fd12dba2924324699a0232e0533f46a6a3916bb0100000000ffffffff026400000000000000160014274ae586ad2035efb4c25049c155f98310d7e106ca16440000000000160014599bcef6387256c6b019030c421b4a4d382fe2600247304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c20121020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f100000000";

    #[test]
    fn decodes_known_transaction_without_error() {
        let result = decode_transaction(KNOWN_TX_HEX.to_string());
        assert!(result.is_ok());
    }

    #[test]
    fn decodes_correct_version() {
        let json = decode_transaction(KNOWN_TX_HEX.to_string()).unwrap();
        assert!(json.contains("\"version\": 2"));
    }

    #[test]
    fn decodes_correct_input_count_and_fields() {
        let json = decode_transaction(KNOWN_TX_HEX.to_string()).unwrap();
        // one input, sequence 0xffffffff = 4294967295, output_index 1
        assert!(json.contains("\"output_index\": 1"));
        assert!(json.contains("\"sequence\": 4294967295"));
        // native SegWit input — empty scriptSig
        assert!(json.contains("\"script_sig\": \"\""));
    }

    #[test]
    fn decodes_correct_output_amounts() {
        let json = decode_transaction(KNOWN_TX_HEX.to_string()).unwrap();
        // first output: 100 sats = 0.000001 BTC
        assert!(json.contains("1e-6") || json.contains("0.000001"));
        // second output: 0.04462282 BTC
        assert!(json.contains("0.04462282"));
    }

    #[test]
    fn decodes_correct_locktime() {
        let json = decode_transaction(KNOWN_TX_HEX.to_string()).unwrap();
        assert!(json.contains("\"lock_time\": 0"));
    }

    #[test]
    fn txid_is_64_hex_characters() {
        let json = decode_transaction(KNOWN_TX_HEX.to_string()).unwrap();
        // crude but effective: pull the transaction_id value out and check its length
        let start = json.find("\"transaction_id\": \"").unwrap() + "\"transaction_id\": \"".len();
        let rest = &json[start..];
        let end = rest.find('"').unwrap();
        let txid = &rest[..end];
        assert_eq!(
            txid.len(),
            64,
            "txid should be 32 bytes hex-encoded (64 chars), got: {txid}"
        );
        assert!(txid.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn rejects_empty_hex_string() {
        let result = decode_transaction(String::new());
        assert!(result.is_err());
    }

    #[test]
    fn rejects_odd_length_hex_string() {
        // one character short of a valid byte pair
        let result = decode_transaction("0200000".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn rejects_non_hex_characters() {
        let result = decode_transaction("zzzz00000000".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn rejects_truncated_transaction() {
        // valid version + segwit marker/flag, but cut off before any
        // input/output data — should fail cleanly, not panic
        let truncated = "0200000000010100"; // deliberately incomplete
        let result = decode_transaction(truncated.to_string());
        assert!(result.is_err());
    }

    #[test]
    fn rejects_missing_segwit_marker_and_flag() {
        // valid version, but next two bytes are NOT 0x00 0x01
        let bad_marker = "02000000".to_string() + "ffff" + "00"; // garbage marker/flag
        let result = decode_transaction(bad_marker);
        assert!(result.is_err());
    }

    #[test]
    fn read_compact_size_handles_single_byte_form() {
        let bytes = [0x05u8, 0xAA, 0xBB];
        let mut slice: &[u8] = &bytes;
        let value = read_compact_size(&mut slice).unwrap();
        assert_eq!(value, 5);
        // cursor should have advanced by exactly 1 byte
        assert_eq!(slice, &[0xAA, 0xBB]);
    }

    #[test]
    fn read_compact_size_handles_0xfd_prefix() {
        // 0xfd followed by little-endian u16: 0x0100 -> 256
        let bytes = [0xfdu8, 0x00, 0x01];
        let mut slice: &[u8] = &bytes;
        let value = read_compact_size(&mut slice).unwrap();
        assert_eq!(value, 256);
        assert!(slice.is_empty());
    }

    #[test]
    fn read_compact_size_handles_0xfe_prefix() {
        // 0xfe followed by little-endian u32: 0x00010000 -> 65536
        let bytes = [0xfeu8, 0x00, 0x00, 0x01, 0x00];
        let mut slice: &[u8] = &bytes;
        let value = read_compact_size(&mut slice).unwrap();
        assert_eq!(value, 65536);
        assert!(slice.is_empty());
    }

    #[test]
    fn read_compact_size_rejects_truncated_input() {
        // 0xfd says "read 2 more bytes" but only 1 remains
        let bytes = [0xfdu8, 0x00];
        let mut slice: &[u8] = &bytes;
        let result = read_compact_size(&mut slice);
        assert!(result.is_err());
    }

    #[test]
    fn read_u32_advances_cursor_correctly() {
        let bytes = [0x02, 0x00, 0x00, 0x00, 0xAA, 0xBB];
        let mut slice: &[u8] = &bytes;
        let value = read_u32(&mut slice).unwrap();
        assert_eq!(value, 2); // little-endian
        assert_eq!(slice, &[0xAA, 0xBB]); // cursor advanced by exactly 4 bytes
    }
}
