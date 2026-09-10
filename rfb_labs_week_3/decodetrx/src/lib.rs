use std::io::Error;
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


#[allow(unused_variables)]
fn read_version(bytes: &mut &[u8]) -> Result<u32, Error> {
    if bytes.len() < 4 {
        return Err(Error::new(std::io::ErrorKind::UnexpectedEof, "Not enough bytes for version"));
    }
    let mut version_bytes = [0u8; 4];
    version_bytes.copy_from_slice(&bytes[0..4]);
    *bytes = &bytes[4..];
    Ok(u32::from_le_bytes(version_bytes))
}


fn read_u64(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    if transaction_bytes.len() < 8 {
        return Err(Error::new(std::io::ErrorKind::UnexpectedEof, "Not enough bytes for u64"));
    }
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&transaction_bytes[0..8]);
    *transaction_bytes = &transaction_bytes[8..];
    Ok(u64::from_le_bytes(bytes))
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    let value = read_u64(transaction_bytes)?;
    Ok(Amount::from_sat(value))
}



fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    if bytes_slice.len() < 4 {
        return Err(Error::new(std::io::ErrorKind::UnexpectedEof, "Not enough bytes for u32"));
    }
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(&bytes_slice[0..4]);
    *bytes_slice = &bytes_slice[4..];
    Ok(u32::from_le_bytes(bytes))
}
  


fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    if transaction_bytes.is_empty() {
        return Err(Error::new(std::io::ErrorKind::UnexpectedEof, "Not enough bytes for compact size"));
    }

    let first = transaction_bytes[0];
    *transaction_bytes = &transaction_bytes[1..];

    match first {
        0x00..=0xfc => Ok(first as u64),
        0xfd => {
            if transaction_bytes.len() < 2 {
                return Err(Error::new(std::io::ErrorKind::UnexpectedEof, "Not enough bytes for u16 in compact size"));
            }
            let mut bytes = [0u8; 2];
            bytes.copy_from_slice(&transaction_bytes[0..2]);
            *transaction_bytes = &transaction_bytes[2..];
            Ok(u16::from_le_bytes(bytes) as u64)
        }
        0xfe => {
            if transaction_bytes.len() < 4 {
                return Err(Error::new(std::io::ErrorKind::UnexpectedEof, "Not enough bytes for u32 in compact size"));
            }
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&transaction_bytes[0..4]);
            *transaction_bytes = &transaction_bytes[4..];
            Ok(u32::from_le_bytes(bytes) as u64)
        }
        0xff => {
            if transaction_bytes.len() < 8 {
                return Err(Error::new(std::io::ErrorKind::UnexpectedEof, "Not enough bytes for u64 in compact size"));
            }
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&transaction_bytes[0..8]);
            *transaction_bytes = &transaction_bytes[8..];
            Ok(u64::from_le_bytes(bytes))
        }
    }
}

fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    if transaction_bytes.len() < 32 {
        return Err(Error::new(std::io::ErrorKind::UnexpectedEof, "Not enough bytes for TXID"));
    }
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&transaction_bytes[0..32]);
    *transaction_bytes = &transaction_bytes[32..];
    Ok(Txid::from_bytes(bytes))
}



#[allow(dead_code)]
fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    read_compact_size(transaction_bytes)
}

fn read_bytes(transaction_bytes: &mut &[u8], len: usize) -> Result<Vec<u8>, Error> {
    if transaction_bytes.len() < len {
        return Err(Error::new(std::io::ErrorKind::UnexpectedEof, "Not enough bytes"));
    }
    let bytes = transaction_bytes[0..len].to_vec();
    *transaction_bytes = &transaction_bytes[len..];
    Ok(bytes)
}

fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u8, Error> {
    if transaction_bytes.is_empty() {
        return Err(Error::new(std::io::ErrorKind::UnexpectedEof, "Not enough bytes for version byte"));
    }
    let byte = transaction_bytes[0];
    *transaction_bytes = &transaction_bytes[1..];
    Ok(byte)
}
// Bitcoin uses little-endian encoding for most of its numeric fields, meaning the least significant byte comes first.

fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    let mut hasher = Sha256::new();
    hasher.update(row_transaction_bytes);
    let first_hash = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.update(&first_hash);
    let final_hash = hasher.finalize();

    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&final_hash[..]);
    Ok(Txid::from_bytes(bytes))
}


pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let bytes_vec = hex::decode(&transaction_hex)?;
    let raw_tx_bytes = bytes_vec.clone();
    let mut bytes = bytes_vec.as_slice();

    let version = read_version(&mut bytes)?;

    let marker = read_version_byte(&mut bytes)?;
    let flag = read_version_byte(&mut bytes)?;

    let is_segwit = marker == 0x00 && flag == 0x01;

    if !is_segwit {
        bytes = raw_tx_bytes.as_slice();
        let version = read_version(&mut bytes)?;
        let in_count = read_compact_size(&mut bytes)?;

        let mut inputs = Vec::new();
        for _ in 0..in_count {
            let txid = read_txid(&mut bytes)?;
            let output_index = read_u32(&mut bytes)?;
            let script_len = read_compact_size(&mut bytes)? as usize;
            let script_sig = read_bytes(&mut bytes, script_len)?;
            let sequence = read_u32(&mut bytes)?;

            inputs.push(Input {
                txid,
                output_index,
                script_sig,
                sequence,
            });
        }

        let out_count = read_compact_size(&mut bytes)?;
        let mut outputs = Vec::new();
        for _ in 0..out_count {
            let amount = read_amount(&mut bytes)?;
            let script_len = read_compact_size(&mut bytes)? as usize;
            let script_pubkey = read_bytes(&mut bytes, script_len)?;

            outputs.push(Output {
                amount,
                script_pubkey,
            });
        }

        let locktime = read_u32(&mut bytes)?;

        let transaction_id = hash_row_transaction(&raw_tx_bytes)?;

        let tx = Transaction {
            transaction_id,
            version,
            inputs,
            outputs,
            lock_time: locktime,
        };

        return Ok(serde_json::to_string_pretty(&tx)?);
    }

    let in_count = read_compact_size(&mut bytes)?;
    let mut inputs = Vec::new();
    for _ in 0..in_count {
        let txid = read_txid(&mut bytes)?;
        let output_index = read_u32(&mut bytes)?;
        let script_len = read_compact_size(&mut bytes)? as usize;
        let script_sig = read_bytes(&mut bytes, script_len)?;
        let sequence = read_u32(&mut bytes)?;

        inputs.push(Input {
            txid,
            output_index,
            script_sig,
            sequence,
        });
    }

    let out_count = read_compact_size(&mut bytes)?;
    let mut outputs = Vec::new();
    for _ in 0..out_count {
        let amount = read_amount(&mut bytes)?;
        let script_len = read_compact_size(&mut bytes)? as usize;
        let script_pubkey = read_bytes(&mut bytes, script_len)?;

        outputs.push(Output {
            amount,
            script_pubkey,
        });
    }

    for _ in 0..in_count {
        let items = read_compact_size(&mut bytes)? as usize;
        for _ in 0..items {
            let len = read_compact_size(&mut bytes)? as usize;
            read_bytes(&mut bytes, len)?;
        }
    }

    let locktime = read_u32(&mut bytes)?;

    let transaction_id = hash_row_transaction(&raw_tx_bytes)?;

    let tx = Transaction {
        transaction_id,
        version,
        inputs,
        outputs,
        lock_time: locktime,
    };

    Ok(serde_json::to_string_pretty(&tx)?)
}