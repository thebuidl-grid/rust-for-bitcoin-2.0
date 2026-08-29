use std::io::{Error, ErrorKind};

use sha2::{Digest, Sha256}; // https://docs.rs/sha2/latest/sha2/
use transaction::{Amount, Input, Output, Transaction, Txid};
mod transaction;

fn unexpected_eof(what: &str) -> Error {
    Error::new(
        ErrorKind::UnexpectedEof,
        format!("not enough bytes to read {what}"),
    )
}

#[allow(dead_code)]
fn read_version(transaction_hex: &str) -> u32 {
    let bytes = hex::decode(transaction_hex).expect("transaction_hex must be valid hex");
    u32::from_le_bytes(
        bytes[0..4]
            .try_into()
            .expect("transaction hex too short for a version"),
    )
}

fn read_u64(transaction_bytes: &mut &[u8]) -> u64 {
    let value = u64::from_le_bytes(
        transaction_bytes[0..8]
            .try_into()
            .expect("not enough bytes for u64"),
    );
    *transaction_bytes = &transaction_bytes[8..];
    value
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    if transaction_bytes.len() < 8 {
        return Err(unexpected_eof("an amount"));
    }
    Ok(Amount::from_sat(read_u64(transaction_bytes)))
}

fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    if bytes_slice.len() < 4 {
        return Err(unexpected_eof("a u32"));
    }
    let value = u32::from_le_bytes(bytes_slice[0..4].try_into().unwrap());
    *bytes_slice = &bytes_slice[4..];
    Ok(value)
}
// Bitcoin uses little-endian encoding for most of its numeric fields, meaning the least significant byte comes first.

fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    if transaction_bytes.is_empty() {
        return Err(unexpected_eof("a compact size prefix"));
    }
    let prefix = transaction_bytes[0];
    *transaction_bytes = &transaction_bytes[1..];

    match prefix {
        0..=0xfc => Ok(prefix as u64),
        0xfd => {
            if transaction_bytes.len() < 2 {
                return Err(unexpected_eof("a 2-byte compact size"));
            }
            let value = u16::from_le_bytes(transaction_bytes[0..2].try_into().unwrap());
            *transaction_bytes = &transaction_bytes[2..];
            Ok(value as u64)
        }
        0xfe => {
            if transaction_bytes.len() < 4 {
                return Err(unexpected_eof("a 4-byte compact size"));
            }
            let value = u32::from_le_bytes(transaction_bytes[0..4].try_into().unwrap());
            *transaction_bytes = &transaction_bytes[4..];
            Ok(value as u64)
        }
        0xff => {
            if transaction_bytes.len() < 8 {
                return Err(unexpected_eof("an 8-byte compact size"));
            }
            let value = u64::from_le_bytes(transaction_bytes[0..8].try_into().unwrap());
            *transaction_bytes = &transaction_bytes[8..];
            Ok(value)
        }
    }
}

fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    if transaction_bytes.len() < 32 {
        return Err(unexpected_eof("a txid"));
    }
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&transaction_bytes[0..32]);
    *transaction_bytes = &transaction_bytes[32..];
    Ok(Txid::from_bytes(bytes))
}

/// Reads a CompactSize-prefixed script and returns its hex encoding.
fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    let length = read_compact_size(transaction_bytes)? as usize;
    if transaction_bytes.len() < length {
        return Err(unexpected_eof("a script"));
    }
    let script_hex = hex::encode(&transaction_bytes[..length]);
    *transaction_bytes = &transaction_bytes[length..];
    Ok(script_hex)
}

fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    read_u32(transaction_bytes)
}

fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    let first_pass = Sha256::digest(row_transaction_bytes);
    let second_pass = Sha256::digest(first_pass);
    Ok(Txid::from_bytes(second_pass.into()))
}

fn encode_compact_size(value: u64) -> Vec<u8> {
    match value {
        0..=0xfc => vec![value as u8],
        0xfd..=0xffff => {
            let mut bytes = vec![0xfd];
            bytes.extend_from_slice(&(value as u16).to_le_bytes());
            bytes
        }
        0x10000..=0xffff_ffff => {
            let mut bytes = vec![0xfe];
            bytes.extend_from_slice(&(value as u32).to_le_bytes());
            bytes
        }
        _ => {
            let mut bytes = vec![0xff];
            bytes.extend_from_slice(&value.to_le_bytes());
            bytes
        }
    }
}

/// Rebuilds the non-witness serialization used to compute a transaction's txid,
/// since SegWit inputs commit their witness data outside of it.
fn legacy_serialize(version: u32, inputs: &[Input], outputs: &[Output], lock_time: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&version.to_le_bytes());

    bytes.extend_from_slice(&encode_compact_size(inputs.len() as u64));
    for input in inputs {
        bytes.extend_from_slice(input.txid.as_bytes());
        bytes.extend_from_slice(&input.output_index.to_le_bytes());
        bytes.extend_from_slice(&encode_compact_size(input.script_sig.len() as u64));
        bytes.extend_from_slice(&input.script_sig);
        bytes.extend_from_slice(&input.sequence.to_le_bytes());
    }

    bytes.extend_from_slice(&encode_compact_size(outputs.len() as u64));
    for output in outputs {
        bytes.extend_from_slice(&output.amount.to_sat().to_le_bytes());
        bytes.extend_from_slice(&encode_compact_size(output.script_pubkey.len() as u64));
        bytes.extend_from_slice(&output.script_pubkey);
    }

    bytes.extend_from_slice(&lock_time.to_le_bytes());
    bytes
}

pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let raw_bytes = hex::decode(transaction_hex.trim())?;
    let mut cursor: &[u8] = &raw_bytes;

    let version = read_version_byte(&mut cursor)?;

    // A SegWit transaction inserts a zero marker byte and a nonzero flag byte
    // right after the version, where a legacy transaction would instead have
    // started counting inputs.
    let is_segwit = cursor.len() >= 2 && cursor[0] == 0x00 && cursor[1] != 0x00;
    if is_segwit {
        cursor = &cursor[2..];
    }

    let input_count = read_compact_size(&mut cursor)?;
    let mut inputs = Vec::with_capacity(input_count as usize);
    for _ in 0..input_count {
        let txid = read_txid(&mut cursor)?;
        let output_index = read_u32(&mut cursor)?;
        let script_sig = hex::decode(read_script_size(&mut cursor)?)?;
        let sequence = read_u32(&mut cursor)?;
        inputs.push(Input {
            txid,
            output_index,
            script_sig,
            sequence,
        });
    }

    let output_count = read_compact_size(&mut cursor)?;
    let mut outputs = Vec::with_capacity(output_count as usize);
    for _ in 0..output_count {
        let amount = read_amount(&mut cursor)?;
        let script_pubkey = hex::decode(read_script_size(&mut cursor)?)?;
        outputs.push(Output {
            amount,
            script_pubkey,
        });
    }

    if is_segwit {
        // Witness data is committed outside of the legacy txid serialization,
        // so it only needs to be consumed here to reach the locktime.
        for _ in 0..input_count {
            let item_count = read_compact_size(&mut cursor)?;
            for _ in 0..item_count {
                read_script_size(&mut cursor)?;
            }
        }
    }

    let lock_time = read_u32(&mut cursor)?;

    let transaction_id =
        hash_row_transaction(&legacy_serialize(version, &inputs, &outputs, lock_time))?;

    let transaction = Transaction {
        transaction_id,
        version,
        inputs,
        outputs,
        lock_time,
    };

    Ok(serde_json::to_string_pretty(&transaction)?)
}
