use std::io::{Read, Error};
// use clap::{Parser, Subcommand};
use clap::{Arg, Command};
use std::fmt;
use sha2::{Sha256, Sha512, Digest}; // https://docs.rs/sha2/latest/sha2/
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


fn take_bytes<'a>(bytes: &mut &'a [u8], n: usize) -> Result<&'a [u8], Error> {
    if bytes.len() < n {
        return Err(Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "not enough bytes remaining in transaction",
        ));
    }
    let (front, rest) = bytes.split_at(n);
    *bytes = rest;
    Ok(front)
}


// #[allow(unused_variables)]
// fn read_version(transaction_hex: &str) -> u32 {
 
// }

fn read_u64(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let bytes = take_bytes(transaction_bytes, 8)?;
    Ok(u64::from_le_bytes(bytes.try_into().unwrap()))
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    let satoshis = read_u64(transaction_bytes)?;
    Ok(Amount::from_sat(satoshis))
}




fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    let bytes = take_bytes(bytes_slice, 4)?;
    Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
}
  


fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let prefix = take_bytes(transaction_bytes, 1)?[0];

    match prefix {
        0x00..=0xfc => Ok(prefix as u64),
        0xfd => {
            let bytes = take_bytes(transaction_bytes, 2)?;
            Ok(u16::from_le_bytes(bytes.try_into().unwrap()) as u64)
        }
        0xfe => {
            let bytes = take_bytes(transaction_bytes, 4)?;
            Ok(u32::from_le_bytes(bytes.try_into().unwrap()) as u64)
        }
        _ => {
            let bytes = take_bytes(transaction_bytes, 8)?;
            Ok(u64::from_le_bytes(bytes.try_into().unwrap()))
        }
    }
}


fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    let bytes = take_bytes(transaction_bytes, 32)?;
    let array: [u8; 32] = bytes.try_into().unwrap();
    Ok(Txid::from_bytes(array))
}



fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    let length = read_compact_size(transaction_bytes)? as usize;
    let script_bytes = take_bytes(transaction_bytes, length)?;
    Ok(hex::encode(script_bytes))
}


// fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {

// }

// Bitcoin uses little-endian encoding for most of its numeric fields, meaning the least significant byte comes first.

fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    let first_hash = Sha256::digest(row_transaction_bytes);
    let second_hash = Sha256::digest(first_hash);
    let bytes: [u8; 32] = second_hash.into();
    Ok(Txid::from_bytes(bytes))
}



pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let raw_bytes = hex::decode(transaction_hex.trim())?;
    let mut cursor: &[u8] = &raw_bytes[..];

    let version = read_u32(&mut cursor)?;
    let after_version = raw_bytes.len() - cursor.len();

    let is_segwit = cursor.first() == Some(&0x00);
    if is_segwit {
        take_bytes(&mut cursor, 2)?; // marker (0x00) + flag (0x01)
    }
    let after_marker_flag = raw_bytes.len() - cursor.len();

    let input_count = read_compact_size(&mut cursor)?;
    let mut inputs = Vec::with_capacity(input_count as usize);
    for _ in 0..input_count {
        inputs.push(Input {
            txid: read_txid(&mut cursor)?,
            output_index: read_u32(&mut cursor)?,
            script_sig: read_script_size(&mut cursor)?,
            sequence: read_u32(&mut cursor)?,
            witness: Vec::new(),
        });
    }

    let output_count = read_compact_size(&mut cursor)?;
    let mut outputs = Vec::with_capacity(output_count as usize);
    for _ in 0..output_count {
        outputs.push(Output {
            amount: read_amount(&mut cursor)?,
            script_pubkey: read_script_size(&mut cursor)?,
        });
    }
    let before_witness = raw_bytes.len() - cursor.len();

    if is_segwit {
        for input in inputs.iter_mut() {
            let item_count = read_compact_size(&mut cursor)?;
            let mut items = Vec::with_capacity(item_count as usize);
            for _ in 0..item_count {
                let item_len = read_compact_size(&mut cursor)? as usize;
                items.push(hex::encode(take_bytes(&mut cursor, item_len)?));
            }
            input.witness = items;
        }
    }
    let after_witness = raw_bytes.len() - cursor.len();

    let lock_time = read_u32(&mut cursor)?;

    let transaction_id = if is_segwit {
        let mut legacy_bytes = Vec::new();
        legacy_bytes.extend_from_slice(&raw_bytes[..after_version]);
        legacy_bytes.extend_from_slice(&raw_bytes[after_marker_flag..before_witness]);
        legacy_bytes.extend_from_slice(&raw_bytes[after_witness..]);
        hash_row_transaction(&legacy_bytes)?
    } else {
        hash_row_transaction(&raw_bytes)?
    };

    let transaction = Transaction { transaction_id, version, inputs, outputs, lock_time };

    Ok(serde_json::to_string_pretty(&transaction)?)
}
