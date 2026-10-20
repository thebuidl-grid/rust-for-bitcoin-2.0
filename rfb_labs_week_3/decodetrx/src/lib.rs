use std::io::Error;
// use clap::{Parser, Subcommand};
use clap::{Arg, Command};
use std::fmt;
use sha2::{Sha256, Sha512, Digest}; // https://docs.rs/sha2/latest/sha2/
use transaction::{Amount, Input, Output, Transaction, Txid};
pub mod transaction;

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
fn read_version(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    read_u32(transaction_bytes)
}


fn read_u64(transaction_bytes: &mut &[u8]) -> u64 {
    if transaction_bytes.len() < 8 {
        return 0;
    }
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&transaction_bytes[..8]);
    *transaction_bytes = &transaction_bytes[8..];
    u64::from_le_bytes(buf)
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    let sats = read_u64(transaction_bytes);
    Ok(Amount::from_sat(sats))
}


fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    if bytes_slice.len() < 4 {
        return Err(Error::new(std::io::ErrorKind::UnexpectedEof, "not enough bytes"));
    }
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&bytes_slice[..4]);
    *bytes_slice = &bytes_slice[4..];
    Ok(u32::from_le_bytes(buf))
}
  


fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    if transaction_bytes.is_empty() {
        return Err(Error::new(std::io::ErrorKind::UnexpectedEof, "not enough bytes"));
    }
    let flag = transaction_bytes[0];
    *transaction_bytes = &transaction_bytes[1..];
    match flag {
        0..=0xfc => Ok(flag as u64),
        0xfd => {
            if transaction_bytes.len() < 2 {
                return Err(Error::new(std::io::ErrorKind::UnexpectedEof, "not enough bytes"));
            }
            let mut buf = [0u8; 2];
            buf.copy_from_slice(&transaction_bytes[..2]);
            *transaction_bytes = &transaction_bytes[2..];
            Ok(u16::from_le_bytes(buf) as u64)
        }
        0xfe => {
            let val = read_u32(transaction_bytes)?;
            Ok(val as u64)
        }
        0xff => {
            let val = read_u64(transaction_bytes);
            Ok(val)
        }
    }
}

fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    if transaction_bytes.len() < 32 {
        return Err(Error::new(std::io::ErrorKind::UnexpectedEof, "not enough bytes"));
    }
    let mut buf = [0u8; 32];
    buf.copy_from_slice(&transaction_bytes[..32]);
    *transaction_bytes = &transaction_bytes[32..];
    Ok(Txid::from_bytes(buf))
}

fn read_bytes<'a>(transaction_bytes: &mut &'a [u8], len: usize) -> Result<&'a [u8], Error> {
    if transaction_bytes.len() < len {
        return Err(Error::new(std::io::ErrorKind::UnexpectedEof, "not enough bytes"));
    }
    let res = &transaction_bytes[..len];
    *transaction_bytes = &transaction_bytes[len..];
    Ok(res)
}

fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    let size = read_compact_size(transaction_bytes)?;
    Ok(size.to_string())
}

fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    read_u32(transaction_bytes)
}

// Bitcoin uses little-endian encoding for most of its numeric fields, meaning the least significant byte comes first.
fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    let hash1 = Sha256::digest(row_transaction_bytes);
    let hash2 = Sha256::digest(&hash1);
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&hash2);
    Ok(Txid::from_bytes(bytes))
}


pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = hex::decode(&transaction_hex)?;
    let mut slice = bytes.as_slice();
    
    let version = read_version(&mut slice)?;
    
    let mut is_segwit = false;
    let mut marker = 0;
    let mut flag = 0;
    
    // Check for SegWit marker
    if !slice.is_empty() && slice[0] == 0 {
        is_segwit = true;
        marker = slice[0];
        if slice.len() > 1 {
            flag = slice[1];
        }
        slice = &slice[2..];
    }
    
    let in_count = read_compact_size(&mut slice)?;
    let mut inputs = Vec::new();
    for _ in 0..in_count {
        let txid = read_txid(&mut slice)?;
        let output_index = read_u32(&mut slice)?;
        let script_len = read_compact_size(&mut slice)?;
        let script_sig = read_bytes(&mut slice, script_len as usize)?.to_vec();
        let sequence = read_u32(&mut slice)?;
        inputs.push(Input {
            txid,
            output_index,
            script_sig,
            sequence,
        });
    }
    
    let out_count = read_compact_size(&mut slice)?;
    let mut outputs = Vec::new();
    for _ in 0..out_count {
        let amount = read_amount(&mut slice)?;
        let script_len = read_compact_size(&mut slice)?;
        let script_pubkey = read_bytes(&mut slice, script_len as usize)?.to_vec();
        outputs.push(Output {
            amount,
            script_pubkey,
        });
    }
    
    if is_segwit {
        for _ in 0..in_count {
            let items = read_compact_size(&mut slice)?;
            for _ in 0..items {
                let len = read_compact_size(&mut slice)?;
                let _item = read_bytes(&mut slice, len as usize)?;
            }
        }
    }
    
    let lock_time = read_u32(&mut slice)?;
    
    // Calculate txid using non-witness data for proper hash
    let mut non_witness_bytes = Vec::new();
    non_witness_bytes.extend_from_slice(&version.to_le_bytes());
    
    // write input count
    let mut write_varint = |bytes: &mut Vec<u8>, val: u64| {
        if val <= 0xfc {
            bytes.push(val as u8);
        } else if val <= 0xffff {
            bytes.push(0xfd);
            bytes.extend_from_slice(&(val as u16).to_le_bytes());
        } else if val <= 0xffffffff {
            bytes.push(0xfe);
            bytes.extend_from_slice(&(val as u32).to_le_bytes());
        } else {
            bytes.push(0xff);
            bytes.extend_from_slice(&val.to_le_bytes());
        }
    };
    
    write_varint(&mut non_witness_bytes, inputs.len() as u64);
    for input in &inputs {
        non_witness_bytes.extend_from_slice(&input.txid.0);
        non_witness_bytes.extend_from_slice(&input.output_index.to_le_bytes());
        write_varint(&mut non_witness_bytes, input.script_sig.len() as u64);
        non_witness_bytes.extend_from_slice(&input.script_sig);
        non_witness_bytes.extend_from_slice(&input.sequence.to_le_bytes());
    }
    
    write_varint(&mut non_witness_bytes, outputs.len() as u64);
    for output in &outputs {
        non_witness_bytes.extend_from_slice(&output.amount.0.to_le_bytes());
        write_varint(&mut non_witness_bytes, output.script_pubkey.len() as u64);
        non_witness_bytes.extend_from_slice(&output.script_pubkey);
    }
    
    non_witness_bytes.extend_from_slice(&lock_time.to_le_bytes());
    
    let transaction_id = hash_row_transaction(&non_witness_bytes)?;
    
    let tx = Transaction {
        transaction_id,
        version,
        inputs,
        outputs,
        lock_time,
    };
    
    Ok(serde_json::to_string_pretty(&tx)?)
}