use std::io::{Read, Error};
// use clap::{Parser, Subcommand};
use clap::{Arg, Command};
use std::fmt;
use sha2::{Sha256, Sha512, Digest}; // https://docs.rs/sha2/latest/sha2/
use transaction::{Amount, Input, Output, Transaction, Txid};
mod transaction;
use serde_json;

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
fn read_version(transaction_hex: &str) -> u32 {
    let transaction_bytes = hex::decode(transaction_hex).expect("invalid hex string");
    let mut slice: &[u8] = &transaction_bytes[..4];
    read_u32(&mut slice).expect("failed to read version")
 
}


fn read_u64(transaction_bytes: &mut &[u8]) -> u64 {
     let mut buffer = [0u8; 8];
    transaction_bytes
        .read_exact(&mut buffer)
        .expect("not enough bytes to read a u64");
    u64::from_le_bytes(buffer)
  
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    let sats = read_u64(transaction_bytes);
    Ok(Amount::from_sat(sats))

}



fn read_u32(bytes_slice: &mut &[u8]) ->Result<u32, Error> {
    let mut buffer = [0u8; 4];
    bytes_slice.read_exact(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}
  


fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let mut prefix = [0u8; 1];
    transaction_bytes.read_exact(&mut prefix)?;

    match prefix[0] {
        0x00..=0xfc => Ok(prefix[0] as u64),
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
    // Read the 32 bytes exactly as they appear on the wire (internal
    // little-endian order). `Txid::serialize` is responsible for reversing
    // them into the conventional display order.
    let mut buffer = [0u8; 32];
    transaction_bytes.read_exact(&mut buffer)?;
    Ok(Txid::from_bytes(buffer))
  
}



fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    let script_len = read_compact_size(transaction_bytes)? as usize;
    let mut script_bytes = vec![0u8; script_len];
    transaction_bytes.read_exact(&mut script_bytes)?;
    Ok(hex::encode(script_bytes))

}

fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
     read_u32(transaction_bytes)

}
// Bitcoin uses little-endian encoding for most of its numeric fields, meaning the least significant byte comes first.

fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    let first_pass = Sha256::digest(row_transaction_bytes);
    let second_pass = Sha256::digest(first_pass);

    let mut txid_bytes = [0u8; 32];
    txid_bytes.copy_from_slice(&second_pass);
    Ok(Txid::from_bytes(txid_bytes))


}


pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
     let owned_bytes = hex::decode(&transaction_hex)?;

    // Hash the whole raw transaction up front, before we start consuming
    // the slice below, since the slice's position advances as we read.
    let transaction_id = hash_row_transaction(&owned_bytes)?;

    let mut bytes: &[u8] = &owned_bytes;

    let version = read_version_byte(&mut bytes)?;

    let input_count = read_compact_size(&mut bytes)?;
    let mut inputs = Vec::with_capacity(input_count as usize);
    for _ in 0..input_count {
        let txid = read_txid(&mut bytes)?;
        let output_index = read_u32(&mut bytes)?;
        let script_sig_hex = read_script_size(&mut bytes)?;
        let script_sig = hex::decode(script_sig_hex)?;
        let sequence = read_u32(&mut bytes)?;

        inputs.push(Input {
            txid,
            output_index,
            script_sig,
            sequence,
        });
    }

    let output_count = read_compact_size(&mut bytes)?;
    let mut outputs = Vec::with_capacity(output_count as usize);
    for _ in 0..output_count {
        let amount = read_amount(&mut bytes)?;
        let script_pubkey_hex = read_script_size(&mut bytes)?;
        let script_pubkey = hex::decode(script_pubkey_hex)?;

        outputs.push(Output {
            amount,
            script_pubkey,
        });
    }

    let lock_time = read_u32(&mut bytes)?;

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
    

