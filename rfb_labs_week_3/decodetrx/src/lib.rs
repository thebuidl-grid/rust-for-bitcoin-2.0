use std::io::{Read, Error};
use clap::{Arg, Command, Parser};
use std::fmt;
use sha2::{Sha256, Sha512, Digest}; // https://docs.rs/sha2/latest/sha2/
use transaction::{Amount, Input, Output, Transaction, Txid};
mod transaction;

#[derive(Parser)]
#[command(name= " Transaction decoder")]
#[command(version= "1.0")]
#[command(about= "Bitcoin Transaction decoder", long_about=None)]
pub struct CLI {
      #[arg(
            required = true,
            help="(string, required) Row Transaction hex"
        )]
    pub transaction_hex: String
}


#[allow(unused_variables)]
fn read_version(transaction_hex: &str) -> u32 {
    let transaction_bytes = hex::decode(transaction_hex).unwrap();
    let mut bytes_slice = &transaction_bytes[..];
    let mut buffer = [0; 4];
    bytes_slice.read(&mut buffer).unwrap();
    u32::from_le_bytes(buffer)
}


fn read_u64(transaction_bytes: &mut &[u8]) -> u64 {
    let mut bytes = [0u8; 8];
    transaction_bytes.read_exact(&mut bytes).unwrap();
    u64::from_le_bytes(bytes)
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    let amount = read_u64(transaction_bytes);
    Ok(Amount::from_sat(amount))
}

fn read_u32(bytes_slice: &mut &[u8]) ->Result<u32, Error> {
    let mut bytes = [0u8; 4];
    bytes_slice.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}


fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let mut compact_size = [0_u8; 1];
    transaction_bytes.read_exact(&mut compact_size)?;

    match compact_size[0] {
        0..=252 => Ok(compact_size[0] as u64),
        253 => {
            let mut buffer = [0; 2];
            transaction_bytes.read_exact(&mut buffer)?;
            Ok(u16::from_le_bytes(buffer) as u64)
        },
        254 => {
            let mut buffer = [0; 4];
            transaction_bytes.read_exact(&mut buffer)?;
            Ok(u32::from_le_bytes(buffer) as u64)
        },
        255 => {
            let mut buffer = [0; 8];
            transaction_bytes.read_exact(&mut buffer)?;
            Ok(u64::from_le_bytes(buffer))
        }

    }
 
}

fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    let mut buffer = [0; 32];
    transaction_bytes.read(&mut buffer)?;
    buffer.reverse();
    Ok(Txid::from_bytes(buffer))
}


fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    let size = read_compact_size(transaction_bytes)?;
    let mut script_bytes = vec![0; size as usize];
    transaction_bytes.read(&mut script_bytes)?;
    Ok(hex::encode(script_bytes))

}

fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    read_u32(transaction_bytes)
}
// Bitcoin uses little-endian encoding for most of its numeric fields, meaning the least significant byte comes first.

fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    let mut hasher = Sha256::new();
    hasher.update(row_transaction_bytes);
    let hash = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.update(hash);
    let hash2 = hasher.finalize();

    Ok(Txid::from_bytes(hash2.into()))

}


pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let transaction_bytes = hex::decode(transaction_hex)
        .map_err(|e| format!("Hex decode error: {:?}", e))?;

    let mut bytes_slice = &transaction_bytes[..];
    let version = read_version_byte(&mut bytes_slice)?;
    let input_count = read_compact_size(&mut bytes_slice)?;
    let mut inputs = vec![];

    for _ in 0..input_count {
        let txid = read_txid(&mut bytes_slice)?;
        let output_index = read_u32(&mut bytes_slice)?;
        let script_sig = hex::decode(read_script_size(&mut bytes_slice)?)?;
        let sequence = read_u32(&mut bytes_slice)?;

        inputs.push(Input {
            txid,
            output_index,
            script_sig,
            sequence,
        })
    }

    let output_count = read_compact_size(&mut bytes_slice)?;
    let mut outputs = vec![];

    for _ in 0..output_count {
        let amount = read_amount(&mut bytes_slice)?;
        let script_pubkey = hex::decode(read_script_size(&mut bytes_slice)?)?;


        outputs.push(Output {
            amount,
            script_pubkey
        })
    }

    let lock_time = read_u32(&mut bytes_slice)?;
    let transaction_id = hash_row_transaction(&transaction_bytes)?;

    let transaction = Transaction {
        transaction_id,
        version,
        inputs,
        outputs,
        lock_time
    };

    Ok(serde_json::to_string_pretty(&transaction)?)

}