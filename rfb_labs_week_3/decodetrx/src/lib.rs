use crate::transaction::{Amount, Input, Output, Transaction, Txid};
use sha2::{Digest, Sha256};
use std::io::Error;

pub mod transaction;

#[allow(dead_code)]
fn read_version(transaction_hex: &str) -> u32 {
    let bytes = hex::decode(&transaction_hex[0..8]).unwrap_or_default();
    if bytes.len() < 4 {
        return 0;
    }
    u32::from_le_bytes(bytes.try_into().unwrap())
}

fn read_u64(transaction_bytes: &mut &[u8]) -> u64 {
    if transaction_bytes.len() < 8 {
        return 0;
    }
    let (buf, rest) = transaction_bytes.split_at(8);
    *transaction_bytes = rest;
    u64::from_le_bytes(buf.try_into().unwrap())
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    let val = read_u64(transaction_bytes);
    Ok(Amount::from_sat(val))
}

fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    if bytes_slice.len() < 4 {
        return Err(Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "Unexpected EOF reading u32",
        ));
    }
    let (buf, rest) = bytes_slice.split_at(4);
    *bytes_slice = rest;
    Ok(u32::from_le_bytes(buf.try_into().unwrap()))
}

fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    if transaction_bytes.is_empty() {
        return Err(Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "Unexpected EOF reading CompactSize",
        ));
    }
    let b = transaction_bytes[0];
    *transaction_bytes = &transaction_bytes[1..];
    match b {
        0..=252 => Ok(b as u64),
        253 => {
            if transaction_bytes.len() < 2 {
                return Err(Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Unexpected EOF reading CompactSize u16",
                ));
            }
            let (buf, rest) = transaction_bytes.split_at(2);
            *transaction_bytes = rest;
            Ok(u16::from_le_bytes(buf.try_into().unwrap()) as u64)
        }
        254 => {
            if transaction_bytes.len() < 4 {
                return Err(Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Unexpected EOF reading CompactSize u32",
                ));
            }
            let (buf, rest) = transaction_bytes.split_at(4);
            *transaction_bytes = rest;
            Ok(u32::from_le_bytes(buf.try_into().unwrap()) as u64)
        }
        255 => {
            if transaction_bytes.len() < 8 {
                return Err(Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Unexpected EOF reading CompactSize u64",
                ));
            }
            let (buf, rest) = transaction_bytes.split_at(8);
            *transaction_bytes = rest;
            Ok(u64::from_le_bytes(buf.try_into().unwrap()))
        }
    }
}

fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    if transaction_bytes.len() < 32 {
        return Err(Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "Unexpected EOF reading Txid",
        ));
    }
    let (buf, rest) = transaction_bytes.split_at(32);
    *transaction_bytes = rest;
    let mut array = [0u8; 32];
    array.copy_from_slice(buf);
    Ok(Txid::from_bytes(array))
}

fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    let len = read_compact_size(transaction_bytes)? as usize;
    if transaction_bytes.len() < len {
        return Err(Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "Unexpected EOF reading script",
        ));
    }
    let (buf, rest) = transaction_bytes.split_at(len);
    *transaction_bytes = rest;
    Ok(hex::encode(buf))
}

fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    read_u32(transaction_bytes)
}

fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    let hash1 = Sha256::digest(row_transaction_bytes);
    let hash2 = Sha256::digest(hash1);
    let mut array = [0u8; 32];
    array.copy_from_slice(&hash2);
    Ok(Txid::from_bytes(array))
}

fn serialize_legacy(version: u32, inputs: &[Input], outputs: &[Output], lock_time: u32) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&version.to_le_bytes());
    buf.extend_from_slice(&encode_compact_size(inputs.len() as u64));
    for input in inputs {
        buf.extend_from_slice(&input.txid.0);
        buf.extend_from_slice(&input.output_index.to_le_bytes());
        buf.extend_from_slice(&encode_compact_size(input.script_sig.len() as u64));
        buf.extend_from_slice(&input.script_sig);
        buf.extend_from_slice(&input.sequence.to_le_bytes());
    }
    buf.extend_from_slice(&encode_compact_size(outputs.len() as u64));
    for output in outputs {
        buf.extend_from_slice(&output.amount.0.to_le_bytes());
        buf.extend_from_slice(&encode_compact_size(output.script_pubkey.len() as u64));
        buf.extend_from_slice(&output.script_pubkey);
    }
    buf.extend_from_slice(&lock_time.to_le_bytes());
    buf
}

fn encode_compact_size(val: u64) -> Vec<u8> {
    let mut buf = Vec::new();
    if val <= 252 {
        buf.push(val as u8);
    } else if val <= 0xffff {
        buf.push(0xfd);
        buf.extend_from_slice(&(val as u16).to_le_bytes());
    } else if val <= 0xffffffff {
        buf.push(0xfe);
        buf.extend_from_slice(&(val as u32).to_le_bytes());
    } else {
        buf.push(0xff);
        buf.extend_from_slice(&val.to_le_bytes());
    }
    buf
}

pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let raw_bytes = hex::decode(transaction_hex.trim())?;
    let mut slice = &raw_bytes[..];

    let version = read_version_byte(&mut slice)?;

    let mut is_segwit = false;
    if slice.len() >= 2 && slice[0] == 0x00 && slice[1] == 0x01 {
        is_segwit = true;
        slice = &slice[2..]; // consume marker and flag
    }

    let input_count = read_compact_size(&mut slice)?;
    let mut inputs = Vec::new();
    for _ in 0..input_count {
        let txid = read_txid(&mut slice)?;
        let output_index = read_u32(&mut slice)?;
        let script_sig_hex = read_script_size(&mut slice)?;
        let script_sig = hex::decode(script_sig_hex)?;
        let sequence = read_u32(&mut slice)?;
        inputs.push(Input {
            txid,
            output_index,
            script_sig,
            sequence,
        });
    }

    let output_count = read_compact_size(&mut slice)?;
    let mut outputs = Vec::new();
    for _ in 0..output_count {
        let amount = read_amount(&mut slice)?;
        let script_pubkey_hex = read_script_size(&mut slice)?;
        let script_pubkey = hex::decode(script_pubkey_hex)?;
        outputs.push(Output {
            amount,
            script_pubkey,
        });
    }

    if is_segwit {
        for _ in 0..input_count {
            let witness_items_count = read_compact_size(&mut slice)?;
            for _ in 0..witness_items_count {
                let item_len = read_compact_size(&mut slice)? as usize;
                if slice.len() < item_len {
                    return Err(Box::new(Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "Unexpected EOF reading witness item",
                    )));
                }
                slice = &slice[item_len..]; // skip witness item bytes
            }
        }
    }

    let lock_time = read_u32(&mut slice)?;

    let legacy_bytes = serialize_legacy(version, &inputs, &outputs, lock_time);
    let transaction_id = hash_row_transaction(&legacy_bytes)?;

    let transaction = Transaction {
        transaction_id,
        version,
        inputs,
        outputs,
        lock_time,
    };

    let json_str = serde_json::to_string_pretty(&transaction)?;
    Ok(json_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_compact_size() {
        let mut slice = &[10u8][..];
        assert_eq!(read_compact_size(&mut slice).unwrap(), 10);
        assert!(slice.is_empty());

        let mut slice = &[253u8, 0x01, 0x01][..];
        assert_eq!(read_compact_size(&mut slice).unwrap(), 257);
        assert!(slice.is_empty());

        let mut slice = &[254u8, 0x01, 0x00, 0x00, 0x00][..];
        assert_eq!(read_compact_size(&mut slice).unwrap(), 1);
        assert!(slice.is_empty());
    }

    #[test]
    fn test_encode_compact_size() {
        assert_eq!(encode_compact_size(10), vec![10]);
        assert_eq!(encode_compact_size(257), vec![253, 0x01, 0x01]);
        assert_eq!(
            encode_compact_size(65537),
            vec![254, 0x01, 0x00, 0x01, 0x00]
        );
    }

    #[test]
    fn test_decode_segwit_transaction() {
        let hex_str = "0200000000010196277c04c986c1ad78c909287fd12dba2924324699a0232e0533f46a6a3916bb0100000000ffffffff026400000000000000160014274ae586ad2035efb4c25049c155f98310d7e106ca16440000000000160014599bcef6387256c6b019030c421b4a4d382fe2600247304402204d94a1e4047ca38a450177ccb6f88585ca147f1939df343d8ac5d962c5f35bb302206f7fa42c21c47ebccdc460393d35c5dfd3b6f0a26cf10fac23d3e6fab71835c20121020cb972a66e3fb1cdcc9efcad060b4457ebec534942700d4af1c0d82a33aa13f100000000";
        let json_str = decode_transaction(hex_str.to_string()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(
            parsed["transaction_id"],
            "be9ea29072566edbc6827e3d9caf1d8c0b57cb0d5e74b95c721c46b3124cbe0b"
        );
        assert_eq!(parsed["version"], 2);
        assert_eq!(parsed["inputs"][0]["output_index"], 1);
        assert_eq!(parsed["outputs"][0]["amount"], 0.000001);
        assert_eq!(parsed["outputs"][1]["amount"], 0.04462282);
        assert_eq!(parsed["lock_time"], 0);
    }
}
