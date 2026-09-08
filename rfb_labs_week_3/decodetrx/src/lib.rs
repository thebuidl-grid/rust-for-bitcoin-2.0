use sha2::{Digest, Sha256}; // https://docs.rs/sha2/latest/sha2/
use std::io::{Error, Read};
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

fn read_u64(bytes_slice: &mut &[u8]) -> Result<u64, Error> {
    let mut buffer = [0; 8];
    bytes_slice.read_exact(&mut buffer)?;
    Ok(u64::from_le_bytes(buffer))
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    let satoshis = read_u64(transaction_bytes)?;
    Ok(Amount::from_sat(satoshis))
}

fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    let mut buffer = [0; 4];
    bytes_slice.read_exact(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let mut first_byte = [0u8; 1];
    transaction_bytes.read_exact(&mut first_byte)?;

    match first_byte[0] {
        0..=0xfc => Ok(first_byte[0] as u64),
        0xfd => {
            let mut buffer = [0u8; 2];
            transaction_bytes.read_exact(&mut buffer)?;
            Ok(u16::from_le_bytes(buffer) as u64)
        }
        0xfe => {
            let mut buffer = [0u8; 4];
            transaction_bytes.read_exact(&mut buffer)?;
            Ok(u32::from_le_bytes(buffer) as u64)
        }
        0xff => {
            let mut buffer = [0u8; 8];
            transaction_bytes.read_exact(&mut buffer)?;
            Ok(u64::from_le_bytes(buffer))
        }
    }
}

fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    let mut buffer = [0u8; 32];
    transaction_bytes.read_exact(&mut buffer)?;
    Ok(Txid::from_bytes(buffer))
}

fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<Vec<u8>, Error> {
    let script_len = read_compact_size(transaction_bytes)?;
    let mut script_bytes = vec![0_u8; script_len as usize];
    transaction_bytes.read_exact(&mut script_bytes)?;
    Ok(script_bytes)
}

// Bitcoin uses little-endian encoding for most of its numeric fields, meaning the least significant byte comes first.

fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    let first_hash = Sha256::digest(row_transaction_bytes);
    let second_hash = Sha256::digest(first_hash);

    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&second_hash);
    bytes.reverse();

    Ok(Txid::from_bytes(bytes))
}

pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let transaction_bytes_owned = hex::decode(&transaction_hex)?;
    let mut cursor: &[u8] = &transaction_bytes_owned;
    let version = read_u32(&mut cursor)?;

    let input_count = read_compact_size(&mut cursor)?;
    let mut inputs = Vec::with_capacity(input_count as usize);
    for _ in 0..input_count {
        let txid = read_txid(&mut cursor)?;
        let output_index = read_u32(&mut cursor)?;
        let script_sig = read_script_size(&mut cursor)?;
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
        let script_pubkey = read_script_size(&mut cursor)?;

        outputs.push(Output {
            amount,
            script_pubkey,
        });
    }

    let lock_time = read_u32(&mut cursor)?;
    let transaction_id = hash_row_transaction(&transaction_bytes_owned)?;

    let transaction = Transaction {
        transaction_id,
        version,
        inputs,
        outputs,
        lock_time,
    };

    let json = serde_json::to_string_pretty(&transaction)?;
    Ok(json)
}
