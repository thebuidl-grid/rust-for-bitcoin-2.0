use std::io::{Read, Error};
use sha2::{Sha256, Digest}; // https://docs.rs/sha2/latest/sha2/
use transaction::{Amount, Input, Output, Transaction, Txid};
mod transaction;

// Bitcoin uses little-endian encoding for most of its numeric fields,
// meaning the least significant byte comes first.

// Convenience helper: version is simply the first 4 bytes of the raw hex.
#[allow(dead_code)]
fn read_version(transaction_hex: &str) -> u32 {
    let transaction_bytes = hex::decode(transaction_hex).unwrap();
    let mut bytes_slice = transaction_bytes.as_slice();
    read_u32(&mut bytes_slice).unwrap()
}

// Reading advances the slice: `Read for &[u8]` re-points the slice past the
// bytes it just handed us, so the next read continues where this one stopped.
fn read_u64(transaction_bytes: &mut &[u8]) -> u64 {
    let mut buffer = [0; 8];
    transaction_bytes.read_exact(&mut buffer).unwrap();
    u64::from_le_bytes(buffer)
}

// An output amount is an 8-byte little-endian count of satoshis.
fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    let mut buffer = [0; 8];
    transaction_bytes.read_exact(&mut buffer)?;
    Ok(Amount::from_sat(u64::from_le_bytes(buffer)))
}

fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    let mut buffer = [0; 4];
    bytes_slice.read_exact(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

// CompactSize: one byte tells us how big the number is, then the number itself.
// Used for input/output counts and script lengths.
fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let marker = read_version_byte(transaction_bytes)?;

    match marker {
        0xfd => {
            let mut buffer = [0; 2];
            transaction_bytes.read_exact(&mut buffer)?;
            Ok(u16::from_le_bytes(buffer) as u64)
        }
        0xfe => Ok(read_u32(transaction_bytes)? as u64),
        0xff => Ok(read_u64(transaction_bytes)),
        _ => Ok(marker as u64), // 0x00..=0xfc is the value itself
    }
}

// The txid of the output being spent, stored in reverse byte order on the wire.
fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    let mut buffer = [0; 32];
    transaction_bytes.read_exact(&mut buffer)?;
    // Kept in wire (little-endian) order; Txid's Serialize flips it for display.
    Ok(Txid::from_bytes(buffer))
}

// A script is a CompactSize length followed by that many bytes.
fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    let script_size = read_compact_size(transaction_bytes)? as usize;
    let mut buffer = vec![0_u8; script_size];
    transaction_bytes.read_exact(&mut buffer)?;
    Ok(hex::encode(buffer))
}

// Reads a single byte (the CompactSize marker).
fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    let mut buffer = [0; 1];
    transaction_bytes.read_exact(&mut buffer)?;
    Ok(buffer[0] as u32)
}

// The transaction id is the double SHA256 of the raw transaction bytes.
fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    let mut hasher = Sha256::new();
    hasher.update(row_transaction_bytes);
    let first_hash = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.update(first_hash);
    let second_hash = hasher.finalize();

    Ok(Txid::from_bytes(second_hash.into()))
}

pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let transaction_bytes = hex::decode(transaction_hex)?;
    let mut bytes_slice = transaction_bytes.as_slice();

    let version = read_u32(&mut bytes_slice)?;

    let input_count = read_compact_size(&mut bytes_slice)?;
    let mut inputs = Vec::new();
    for _ in 0..input_count {
        inputs.push(Input {
            txid: read_txid(&mut bytes_slice)?,
            output_index: read_u32(&mut bytes_slice)?,
            script_sig: read_script_size(&mut bytes_slice)?,
            sequence: read_u32(&mut bytes_slice)?,
        });
    }

    let output_count = read_compact_size(&mut bytes_slice)?;
    let mut outputs = Vec::new();
    for _ in 0..output_count {
        outputs.push(Output {
            amount: read_amount(&mut bytes_slice)?,
            script_pubkey: read_script_size(&mut bytes_slice)?,
        });
    }

    let lock_time = read_u32(&mut bytes_slice)?;

    let transaction = Transaction {
        transaction_id: hash_row_transaction(&transaction_bytes)?,
        version,
        inputs,
        outputs,
        lock_time,
    };

    Ok(serde_json::to_string_pretty(&transaction)?)
}
