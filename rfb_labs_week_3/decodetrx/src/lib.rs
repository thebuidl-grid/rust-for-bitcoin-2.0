use std::io::{Read, Error};
use sha2::{Sha256, Digest};
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

/// Helper function to parse raw script bytes (Vec<u8>)
#[allow(dead_code)]
fn read_script(transaction_bytes: &mut &[u8]) -> Result<Vec<u8>, Error> {
    let script_len = read_compact_size(transaction_bytes)? as usize;
    let mut script_buf = vec![0u8; script_len];
    transaction_bytes.read_exact(&mut script_buf)?;
    Ok(script_buf)
}

/// Standalone version reader from raw hex string if needed
#[allow(unused_variables)]
#[allow(dead_code)]
fn read_version(transaction_hex: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let bytes = hex::decode(&transaction_hex[..8])?;
    let mut slice = bytes.as_slice();
    Ok(read_u32(&mut slice)?)
}

fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    let mut buf = [0u8; 4];
    bytes_slice.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_u64(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let mut buf = [0u8; 8];
    transaction_bytes.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    let sats = read_u64(transaction_bytes)?;
    Ok(Amount::from_sat(sats))
}

fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let mut buf = [0u8; 1];
    transaction_bytes.read_exact(&mut buf)?;
    
    match buf[0] {
        0x00..=0xFC => Ok(buf[0] as u64),
        0xFD => {
            let mut buf = [0u8; 2];
            transaction_bytes.read_exact(&mut buf)?;
            Ok(u16::from_le_bytes(buf) as u64)
        }
        0xFE => {
            let mut buf = [0u8; 4];
            transaction_bytes.read_exact(&mut buf)?;
            Ok(u32::from_le_bytes(buf) as u64)
        }
        0xFF => read_u64(transaction_bytes),
    }
}

fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    let mut buf = [0u8; 32];
    transaction_bytes.read_exact(&mut buf)?;
    // Txids in Bitcoin protocol are serialized in reverse order (little-endian hash)
    buf.reverse(); 
    Ok(Txid::from_bytes(buf))
}

/// Reads the script size length and converts it to its hex string representation
#[allow(dead_code)]
fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    let script_len = read_compact_size(transaction_bytes)?;
    Ok(format!("{:x}", script_len))
}

/// Reads a 4-byte version field from the byte stream
#[allow(dead_code)]
fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    read_u32(transaction_bytes)
}

// Bitcoin uses little-endian encoding for most of its numeric fields, meaning the least significant byte comes first.

fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    // First SHA-256 pass
    let first_hash = Sha256::digest(row_transaction_bytes);
    // Second SHA-256 pass
    let second_hash = Sha256::digest(first_hash);
    
    let mut hash_bytes: [u8; 32] = second_hash.into();
    // Reverse bytes to output little-endian display format
    hash_bytes.reverse();
    
    Ok(Txid::from_bytes(hash_bytes))
}


pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let clean_hex = transaction_hex.trim();
    
    eprintln!("[DEBUG] Clean Hex Length: {} chars", clean_hex.len());
    let raw_bytes = hex::decode(clean_hex)?;
    let mut bytes_slice = raw_bytes.as_slice();
    eprintln!("[DEBUG] Total Bytes to parse: {}", bytes_slice.len());

    let txid = hash_row_transaction(&raw_bytes)?;
    let version = read_u32(&mut bytes_slice)?;
    eprintln!("[DEBUG] -> Version: {}", version);

    let input_count = read_compact_size(&mut bytes_slice)?;
    eprintln!("[DEBUG] -> Inputs Count: {}", input_count);

    let mut inputs = Vec::with_capacity(input_count as usize);
    for i in 0..input_count {
        let prev_txid = read_txid(&mut bytes_slice)?;
        let output_index = read_u32(&mut bytes_slice)?;
        let script_sig = read_script(&mut bytes_slice)?;
        let sequence = read_u32(&mut bytes_slice)?;

        eprintln!("[DEBUG]   Input #{}: index={}, script_sig_len={} bytes", i, output_index, script_sig.len());

        inputs.push(Input {
            txid: prev_txid,
            output_index,
            script_sig,
            sequence,
        });
    }

    let output_count = read_compact_size(&mut bytes_slice)?;
    eprintln!("[DEBUG] -> Outputs Count: {}", output_count);

    let mut outputs = Vec::with_capacity(output_count as usize);
    for i in 0..output_count {
        let amount = read_amount(&mut bytes_slice)?;
        let script_pubkey = read_script(&mut bytes_slice)?;

        eprintln!("[DEBUG]   Output #{}: amount={:?}, script_pubkey_len={} bytes", i, amount, script_pubkey.len());

        outputs.push(Output {
            amount,
            script_pubkey,
        });
    }

    let lock_time = read_u32(&mut bytes_slice)?;
    eprintln!("[DEBUG] -> Lock Time: {}", lock_time);
    eprintln!("[DEBUG] -> Remaining unparsed bytes at end: {}", bytes_slice.len());

    let tx = Transaction {
        transaction_id: txid,
        version,
        inputs,
        outputs,
        lock_time,
    };

    let json_output = serde_json::to_string_pretty(&tx)?;
    Ok(json_output)
}