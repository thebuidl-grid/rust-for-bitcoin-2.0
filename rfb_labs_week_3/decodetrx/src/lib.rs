use std::io::{Error, ErrorKind};
use sha2::{Sha256, Digest};
use transaction::{Amount, Input, Output, Transaction, Txid};
mod transaction;

#[allow(dead_code)]
fn read_version(transaction_hex: &str) -> u32 {
    let bytes = hex::decode(&transaction_hex[0..8]).unwrap();
    u32::from_le_bytes(bytes.try_into().unwrap())
}

fn read_u64(transaction_bytes: &mut &[u8]) -> u64 {
    let (value, rest) = transaction_bytes.split_at(8);
    *transaction_bytes = rest;
    u64::from_le_bytes(value.try_into().unwrap())
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    Ok(Amount::from_sat(read_u64(transaction_bytes)))
}

fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    if bytes_slice.len() < 4 {
        return Err(Error::new(ErrorKind::UnexpectedEof, "not enough bytes for u32"));
    }
    let (value, rest) = bytes_slice.split_at(4);
    *bytes_slice = rest;
    Ok(u32::from_le_bytes(value.try_into().unwrap()))
}

fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    if transaction_bytes.is_empty() {
        return Err(Error::new(ErrorKind::UnexpectedEof, "not enough bytes for compact size"));
    }
    let (prefix, rest) = transaction_bytes.split_at(1);
    *transaction_bytes = rest;
    match prefix[0] {
        0x00..=0xfc => Ok(prefix[0] as u64),
        0xfd => {
            if transaction_bytes.len() < 2 {
                return Err(Error::new(ErrorKind::UnexpectedEof, "not enough bytes for compact size"));
            }
            let (value, rest) = transaction_bytes.split_at(2);
            *transaction_bytes = rest;
            Ok(u16::from_le_bytes(value.try_into().unwrap()) as u64)
        }
        0xfe => Ok(read_u32(transaction_bytes)? as u64),
        _ => Ok(read_u64(transaction_bytes)),
    }
}

fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    if transaction_bytes.len() < 32 {
        return Err(Error::new(ErrorKind::UnexpectedEof, "not enough bytes for txid"));
    }
    let (txid_bytes, rest) = transaction_bytes.split_at(32);
    *transaction_bytes = rest;
    let mut reversed: [u8; 32] = txid_bytes.try_into().unwrap();
    reversed.reverse();
    Ok(Txid::from_bytes(reversed))
}

fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    let script_len = read_compact_size(transaction_bytes)? as usize;
    if transaction_bytes.len() < script_len {
        return Err(Error::new(ErrorKind::UnexpectedEof, "not enough bytes for script"));
    }
    let (script, rest) = transaction_bytes.split_at(script_len);
    *transaction_bytes = rest;
    Ok(hex::encode(script))
}

fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    read_u32(transaction_bytes)
}

fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    let first_hash = Sha256::digest(row_transaction_bytes);
    let second_hash = Sha256::digest(first_hash);
    let mut reversed: [u8; 32] = second_hash.into();
    reversed.reverse();
    Ok(Txid::from_bytes(reversed))
}

pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = hex::decode(transaction_hex.trim())?;
    let mut slice: &[u8] = &bytes;

    let version = read_version_byte(&mut slice)?;
    let version_bytes = &bytes[0..4];

    if slice.len() < 2 || slice[0] != 0x00 || slice[1] != 0x01 {
        return Err("not a SegWit transaction (missing marker/flag)".into());
    }
    slice = &slice[2..];

    let inputs_start = bytes.len() - slice.len();

    let input_count = read_compact_size(&mut slice)?;
    let mut inputs = Vec::with_capacity(input_count as usize);
    for _ in 0..input_count {
        let txid = read_txid(&mut slice)?;
        let output_index = read_u32(&mut slice)?;
        let script_sig = hex::decode(read_script_size(&mut slice)?)?;
        let sequence = read_u32(&mut slice)?;
        inputs.push(Input { txid, output_index, script_sig, sequence, witness: Vec::new() });
    }

    let output_count = read_compact_size(&mut slice)?;
    let mut outputs = Vec::with_capacity(output_count as usize);
    for _ in 0..output_count {
        let amount = read_amount(&mut slice)?;
        let script_pubkey = hex::decode(read_script_size(&mut slice)?)?;
        outputs.push(Output { amount, script_pubkey });
    }

    let outputs_end = bytes.len() - slice.len();

    for input in inputs.iter_mut() {
        let witness_count = read_compact_size(&mut slice)?;
        let mut witness = Vec::with_capacity(witness_count as usize);
        for _ in 0..witness_count {
            witness.push(hex::decode(read_script_size(&mut slice)?)?);
        }
        input.witness = witness;
    }

    let lock_time = read_u32(&mut slice)?;
    let lock_time_bytes = &bytes[bytes.len() - 4..];

    // txid is the double-SHA256 of the legacy (non-witness) serialization
    let mut legacy_bytes = Vec::with_capacity(8 + outputs_end - inputs_start);
    legacy_bytes.extend_from_slice(version_bytes);
    legacy_bytes.extend_from_slice(&bytes[inputs_start..outputs_end]);
    legacy_bytes.extend_from_slice(lock_time_bytes);
    let transaction_id = hash_row_transaction(&legacy_bytes)?;

    let transaction = Transaction { transaction_id, version, inputs, outputs, lock_time };

    Ok(serde_json::to_string_pretty(&transaction)?)
}
