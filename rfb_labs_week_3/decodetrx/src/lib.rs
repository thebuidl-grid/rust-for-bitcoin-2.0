use std::io::{Error, ErrorKind};
use sha2::{Digest, Sha256};
use transaction::{Amount, Input, Output, Transaction, Txid};
mod transaction;

// #[derive(Parser)]
// #[command(name= " Transaction decoder")]
// #[command(version= "1.0")]
// #[command(about= "Bitcoin Transaction decoder", long_about=None)]
// struct CLI {
//       #[arg(
//             required = true,
//             help="(string, required) Row Transaction hex"
//         )]
//     transaction_hex: String
// }

/// Turn a hex string into raw bytes.
fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>, Error> {
    if hex_str.len() % 2 != 0 {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "hex string must have an even number of characters",
        ));
    }

    let mut bytes = Vec::with_capacity(hex_str.len() / 2);
    for i in (0..hex_str.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex_str[i..i + 2], 16)
            .map_err(|e| Error::new(ErrorKind::InvalidInput, e))?;
        bytes.push(byte);
    }
    Ok(bytes)
}

/// Turn raw bytes into a lowercase hex string.
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{:02x}", byte)).collect()
}

/// Pull `n` bytes off the front of the slice, advancing it past what was read.
fn take_bytes<'a>(transaction_bytes: &mut &'a [u8], n: usize) -> Result<&'a [u8], Error> {
    if transaction_bytes.len() < n {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            format!(
                "expected {} more byte(s), only {} left",
                n,
                transaction_bytes.len()
            ),
        ));
    }
    let (head, tail) = transaction_bytes.split_at(n);
    *transaction_bytes = tail;
    Ok(head)
}

// Bitcoin uses little-endian encoding for most of its numeric fields, meaning the least significant byte comes first.

#[allow(dead_code)]
fn read_version(transaction_hex: &str) -> u32 {
    let version_hex = &transaction_hex[0..8];
    let bytes = hex_to_bytes(version_hex).expect("valid version hex");
    u32::from_le_bytes(bytes.try_into().expect("4 bytes"))
}

fn read_u64(transaction_bytes: &mut &[u8]) -> u64 {
    let bytes = take_bytes(transaction_bytes, 8).expect("8 bytes for u64");
    u64::from_le_bytes(bytes.try_into().expect("8 bytes"))
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    let satoshis = read_u64(transaction_bytes);
    Ok(Amount::from_sat(satoshis))
}

fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    let bytes = take_bytes(bytes_slice, 4)?;
    Ok(u32::from_le_bytes(bytes.try_into().expect("4 bytes")))
}

fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let prefix = take_bytes(transaction_bytes, 1)?[0];

    let value = match prefix {
        0xfd => {
            let bytes = take_bytes(transaction_bytes, 2)?;
            u16::from_le_bytes(bytes.try_into().expect("2 bytes")) as u64
        }
        0xfe => {
            let bytes = take_bytes(transaction_bytes, 4)?;
            u32::from_le_bytes(bytes.try_into().expect("4 bytes")) as u64
        }
        0xff => {
            let bytes = take_bytes(transaction_bytes, 8)?;
            u64::from_le_bytes(bytes.try_into().expect("8 bytes"))
        }
        small => small as u64,
    };

    Ok(value)
}

fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    let bytes = take_bytes(transaction_bytes, 32)?;
    let mut txid_bytes = [0u8; 32];
    txid_bytes.copy_from_slice(bytes);
    Ok(Txid::from_bytes(txid_bytes))
}

#[allow(dead_code)]
fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    let script_len = read_compact_size(transaction_bytes)? as usize;
    let script_bytes = take_bytes(transaction_bytes, script_len)?;
    Ok(bytes_to_hex(script_bytes))
}

fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    read_u32(transaction_bytes)
}

fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    let first_hash = Sha256::digest(row_transaction_bytes);
    let second_hash = Sha256::digest(first_hash);

    let mut txid_bytes = [0u8; 32];
    txid_bytes.copy_from_slice(&second_hash);
    Ok(Txid::from_bytes(txid_bytes))
}

pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let raw_transaction_bytes = hex_to_bytes(&transaction_hex)?;

    // The txid is the double-SHA256 hash of the raw (non-witness) transaction bytes.
    let transaction_id = hash_row_transaction(&raw_transaction_bytes)?;

    let mut cursor: &[u8] = &raw_transaction_bytes;

    let version = read_version_byte(&mut cursor)?;

    let input_count = read_compact_size(&mut cursor)?;
    let mut inputs = Vec::with_capacity(input_count as usize);
    for _ in 0..input_count {
        let txid = read_txid(&mut cursor)?;
        let output_index = read_u32(&mut cursor)?;
        let script_sig_len = read_compact_size(&mut cursor)? as usize;
        let script_sig = take_bytes(&mut cursor, script_sig_len)?.to_vec();
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
        let script_pubkey_len = read_compact_size(&mut cursor)? as usize;
        let script_pubkey = take_bytes(&mut cursor, script_pubkey_len)?.to_vec();

        outputs.push(Output {
            amount,
            script_pubkey,
        });
    }

    let lock_time = read_u32(&mut cursor)?;

    let transaction = Transaction {
        transaction_id,
        version,
        inputs,
        outputs,
        lock_time,
    };

    Ok(serde_json::to_string_pretty(&transaction)?)
}
