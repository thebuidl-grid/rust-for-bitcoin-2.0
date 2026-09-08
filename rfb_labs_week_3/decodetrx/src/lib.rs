use std::io::Read;

use error::DecodeError;
use sha2::{Digest, Sha256};

pub mod error;
mod transaction;

pub use transaction::{Amount, Input, Output, Transaction, TransactionFormat, Txid};

#[allow(unused_variables)]
pub fn read_version(transaction_hex: &str) -> Result<u32, DecodeError> {
    let transaction_bytes = hex::decode(transaction_hex)?;
    let mut cursor = transaction_bytes.as_slice();
    read_u32(&mut cursor)
}

pub fn read_u64(transaction_bytes: &mut &[u8]) -> Result<u64, DecodeError> {
    let mut buffer = [0_u8; 8];
    transaction_bytes.read_exact(&mut buffer)?;
    Ok(u64::from_le_bytes(buffer))
}

pub fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, DecodeError> {
    let satoshis = read_u64(transaction_bytes)?;
    Ok(Amount::from_sat(satoshis))
}

pub fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, DecodeError> {
    let mut buffer = [0_u8; 4];
    bytes_slice.read_exact(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

pub fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, DecodeError> {
    let prefix = read_u8(transaction_bytes)?;

    match prefix {
        0x00..=0xfc => Ok(prefix as u64),
        0xfd => {
            let value = read_u16(transaction_bytes)? as u64;

            if value < 0xfd {
                return Err(DecodeError::InvalidCompactSize);
            }

            Ok(value)
        }
        0xfe => {
            let value = read_u32(transaction_bytes)? as u64;

            if value <= u16::MAX as u64 {
                return Err(DecodeError::InvalidCompactSize);
            }

            Ok(value)
        }
        0xff => {
            let value = read_u64(transaction_bytes)?;

            if value <= u32::MAX as u64 {
                return Err(DecodeError::InvalidCompactSize);
            }

            Ok(value)
        }
    }
}

pub fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, DecodeError> {
    let mut bytes = [0_u8; 32];
    transaction_bytes.read_exact(&mut bytes)?;
    Ok(Txid::from_bytes(bytes))
}

pub fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<Vec<u8>, DecodeError> {
    let script_size = read_compact_size(transaction_bytes)?;
    let script_size =
        usize::try_from(script_size).map_err(|_| DecodeError::ScriptLengthTooLarge(script_size))?;

    read_bytes(transaction_bytes, script_size)
}

pub fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, DecodeError> {
    read_u32(transaction_bytes)
}
// Bitcoin uses little-endian encoding for most of its numeric fields, meaning the least significant byte comes first.

pub fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, DecodeError> {
    let first_hash = Sha256::digest(row_transaction_bytes);
    let second_hash = Sha256::digest(first_hash);
    let mut txid_bytes = [0_u8; 32];
    txid_bytes.copy_from_slice(&second_hash);

    Ok(Txid::from_bytes(txid_bytes))
}

pub fn decode_transaction(transaction_hex: String) -> Result<String, DecodeError> {
    let transaction = parse_transaction(&transaction_hex)?;
    Ok(serde_json::to_string_pretty(&transaction)?)
}

// Additional parsing helpers (not included in the starter code).

pub fn read_bytes(transaction_bytes: &mut &[u8], length: usize) -> Result<Vec<u8>, DecodeError> {
    let mut bytes = vec![0_u8; length];
    transaction_bytes.read_exact(&mut bytes)?;
    Ok(bytes)
}

pub fn read_u8(transaction_bytes: &mut &[u8]) -> Result<u8, DecodeError> {
    let mut buffer = [0_u8; 1];
    transaction_bytes.read_exact(&mut buffer)?;
    Ok(u8::from_le_bytes(buffer))
}

pub fn read_u16(transaction_bytes: &mut &[u8]) -> Result<u16, DecodeError> {
    let mut buffer = [0_u8; 2];
    transaction_bytes.read_exact(&mut buffer)?;
    Ok(u16::from_le_bytes(buffer))
}

pub fn read_transaction_format(
    transaction_bytes: &mut &[u8],
) -> Result<TransactionFormat, DecodeError> {
    if transaction_bytes.first() != Some(&0x00) {
        return Ok(TransactionFormat::Legacy);
    }

    let marker = read_u8(transaction_bytes)?;
    let flag = read_u8(transaction_bytes)?;

    if marker != 0x00 || flag != 0x01 {
        return Err(DecodeError::InvalidSegwitMarker);
    }

    Ok(TransactionFormat::Segwit)
}

pub fn parse_transaction(transaction_hex: &str) -> Result<Transaction, DecodeError> {
    let raw_transaction = hex::decode(transaction_hex)?;
    let total_length = raw_transaction.len();
    let mut cursor = raw_transaction.as_slice();

    let version = read_version_byte(&mut cursor)?;
    let format = read_transaction_format(&mut cursor)?;
    let base_body_start = total_length - cursor.len();

    let mut inputs = read_inputs(&mut cursor)?;
    let outputs = read_outputs(&mut cursor)?;
    let base_body_end = total_length - cursor.len();

    if format == TransactionFormat::Segwit {
        read_witnesses(&mut cursor, &mut inputs)?;
    }

    let lock_time_start = total_length - cursor.len();
    let lock_time = read_u32(&mut cursor)?;

    if !cursor.is_empty() {
        return Err(DecodeError::TrailingBytes(cursor.len()));
    }

    let transaction_id = match format {
        TransactionFormat::Legacy => hash_row_transaction(&raw_transaction)?,
        TransactionFormat::Segwit => {
            let mut non_witness_bytes = Vec::with_capacity(
                4 + (base_body_end - base_body_start) + std::mem::size_of::<u32>(),
            );
            non_witness_bytes.extend_from_slice(&raw_transaction[..4]);
            non_witness_bytes.extend_from_slice(&raw_transaction[base_body_start..base_body_end]);
            non_witness_bytes
                .extend_from_slice(&raw_transaction[lock_time_start..lock_time_start + 4]);
            hash_row_transaction(&non_witness_bytes)?
        }
    };

    Ok(Transaction {
        transaction_id,
        version,
        inputs,
        outputs,
        lock_time,
    })
}

fn read_inputs(transaction_bytes: &mut &[u8]) -> Result<Vec<Input>, DecodeError> {
    let input_count = read_compact_size(transaction_bytes)?;
    let mut inputs = Vec::new();

    for _ in 0..input_count {
        let txid = read_txid(transaction_bytes)?;
        let output_index = read_u32(transaction_bytes)?;
        let script_sig = read_script_size(transaction_bytes)?;
        let sequence = read_u32(transaction_bytes)?;

        inputs.push(Input {
            txid,
            output_index,
            script_sig,
            sequence,
            witness: Vec::new(),
        });
    }

    Ok(inputs)
}

fn read_outputs(transaction_bytes: &mut &[u8]) -> Result<Vec<Output>, DecodeError> {
    let output_count = read_compact_size(transaction_bytes)?;
    let mut outputs = Vec::new();

    for _ in 0..output_count {
        let amount = read_amount(transaction_bytes)?;
        let script_pubkey = read_script_size(transaction_bytes)?;

        outputs.push(Output {
            amount,
            script_pubkey,
        });
    }

    Ok(outputs)
}

fn read_witnesses(transaction_bytes: &mut &[u8], inputs: &mut [Input]) -> Result<(), DecodeError> {
    for input in inputs {
        let item_count = read_compact_size(transaction_bytes)?;
        let mut witness = Vec::new();

        for _ in 0..item_count {
            witness.push(read_script_size(transaction_bytes)?);
        }

        input.witness = witness;
    }

    Ok(())
}
