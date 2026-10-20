#![allow(unused_imports)]

use std::io::{Error, Read};
// use clap::{Parser, Subcommand};
use clap::{Arg, Command};
use sha2::{Digest, Sha256, Sha512}; // https://docs.rs/sha2/latest/sha2/
use std::fmt;
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

#[allow(unused_variables)]
pub fn read_version(transaction_hex: &str) -> u32 {
    let bytes = hex::decode(transaction_hex.trim()).unwrap_or_default();
    if bytes.len() < 4 {
        return 0;
    }
    let mut cursor = &bytes[0..4];
    read_u32(&mut cursor).unwrap_or(0)
}

pub fn read_u64(transaction_bytes: &mut &[u8]) -> u64 {
    let mut buf = [0u8; 8];
    if transaction_bytes.read_exact(&mut buf).is_err() {
        return 0;
    }
    u64::from_le_bytes(buf)
}

pub fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    let mut buf = [0u8; 8];
    transaction_bytes.read_exact(&mut buf)?;
    Ok(Amount::from_sat(u64::from_le_bytes(buf)))
}

pub fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    let mut buf = [0u8; 4];
    bytes_slice.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

pub fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let mut first = [0u8; 1];
    transaction_bytes.read_exact(&mut first)?;
    match first[0] {
        0x00..=0xfc => Ok(first[0] as u64),
        0xfd => {
            let mut buf = [0u8; 2];
            transaction_bytes.read_exact(&mut buf)?;
            Ok(u16::from_le_bytes(buf) as u64)
        }
        0xfe => {
            let mut buf = [0u8; 4];
            transaction_bytes.read_exact(&mut buf)?;
            Ok(u32::from_le_bytes(buf) as u64)
        }
        0xff => {
            let mut buf = [0u8; 8];
            transaction_bytes.read_exact(&mut buf)?;
            Ok(u64::from_le_bytes(buf))
        }
    }
}

pub fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    let mut buf = [0u8; 32];
    transaction_bytes.read_exact(&mut buf)?;
    Ok(Txid::from_bytes(buf))
}

pub fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    let len = read_compact_size(transaction_bytes)? as usize;
    let mut buf = vec![0u8; len];
    transaction_bytes.read_exact(&mut buf)?;
    Ok(hex::encode(buf))
}

pub fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    read_u32(transaction_bytes)
}

// Bitcoin uses little-endian encoding for most of its numeric fields, meaning the least significant byte comes first.

pub fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    let first_hash = Sha256::digest(row_transaction_bytes);
    let second_hash = Sha256::digest(first_hash);
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&second_hash);
    Ok(Txid::from_bytes(bytes))
}

pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let clean_hex = transaction_hex.trim();
    let raw_bytes = hex::decode(clean_hex)?;
    let mut cursor = raw_bytes.as_slice();

    let version = read_version_byte(&mut cursor)?;

    let mut is_segwit = false;
    if cursor.len() >= 2 && cursor[0] == 0x00 && cursor[1] != 0x00 {
        is_segwit = true;
        let mut marker_flag = [0u8; 2];
        cursor.read_exact(&mut marker_flag)?;
    }

    let mut legacy_bytes = Vec::new();
    legacy_bytes.extend_from_slice(&version.to_le_bytes());

    let input_count = read_compact_size(&mut cursor)?;
    encode_compact_size(input_count, &mut legacy_bytes);

    let mut inputs = Vec::new();
    for _ in 0..input_count {
        let prev_txid = read_txid(&mut cursor)?;
        legacy_bytes.extend_from_slice(&prev_txid.to_bytes());

        let vout = read_u32(&mut cursor)?;
        legacy_bytes.extend_from_slice(&vout.to_le_bytes());

        let script_len = read_compact_size(&mut cursor)?;
        encode_compact_size(script_len, &mut legacy_bytes);

        let mut script_sig = vec![0u8; script_len as usize];
        cursor.read_exact(&mut script_sig)?;
        legacy_bytes.extend_from_slice(&script_sig);

        let sequence = read_u32(&mut cursor)?;
        legacy_bytes.extend_from_slice(&sequence.to_le_bytes());

        inputs.push(Input {
            txid: prev_txid,
            output_index: vout,
            script_sig,
            sequence,
        });
    }

    let output_count = read_compact_size(&mut cursor)?;
    encode_compact_size(output_count, &mut legacy_bytes);

    let mut outputs = Vec::new();
    for _ in 0..output_count {
        let amount = read_amount(&mut cursor)?;
        legacy_bytes.extend_from_slice(&amount.0.to_le_bytes());

        let script_len = read_compact_size(&mut cursor)?;
        encode_compact_size(script_len, &mut legacy_bytes);

        let mut script_pubkey = vec![0u8; script_len as usize];
        cursor.read_exact(&mut script_pubkey)?;
        legacy_bytes.extend_from_slice(&script_pubkey);

        outputs.push(Output {
            amount,
            script_pubkey,
        });
    }

    if is_segwit {
        for _ in 0..input_count {
            let witness_count = read_compact_size(&mut cursor)?;
            for _ in 0..witness_count {
                let item_len = read_compact_size(&mut cursor)? as usize;
                let mut item_bytes = vec![0u8; item_len];
                cursor.read_exact(&mut item_bytes)?;
            }
        }
    }

    let lock_time = read_u32(&mut cursor)?;
    legacy_bytes.extend_from_slice(&lock_time.to_le_bytes());

    let transaction_id = hash_row_transaction(&legacy_bytes)?;

    let tx = Transaction {
        transaction_id,
        version,
        inputs,
        outputs,
        lock_time,
    };

    let json_output = serde_json::to_string_pretty(&tx)?;
    Ok(json_output)
}

fn encode_compact_size(val: u64, out: &mut Vec<u8>) {
    if val <= 0xfc {
        out.push(val as u8);
    } else if val <= 0xffff {
        out.push(0xfd);
        out.extend_from_slice(&(val as u16).to_le_bytes());
    } else if val <= 0xffffffff {
        out.push(0xfe);
        out.extend_from_slice(&(val as u32).to_le_bytes());
    } else {
        out.push(0xff);
        out.extend_from_slice(&val.to_le_bytes());
    }
}
