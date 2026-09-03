use std::io::{Read, Error};
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


#[allow(unused_variables)]
fn read_version(transaction_hex: &str) -> u32 {
 
}


fn read_u64(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let mut buffer = [0u8; 8];
    transaction_bytes.read_exact(&mut buffer)?;

    Ok(u64::from_le_bytes(buffer))
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    let mut buffer = [0u8; 8];
    transaction_bytes.read_exact(&mut buffer)?;


    let satoshis = u64::from_le_bytes(buffer);
    Ok(Amount::from_sat(satoshis))
}


fn read_u32(bytes_slice: &mut &[u8]) ->Result<u32, Error> {
    let mut buffer = [0u8; 4];
    bytes_slice.read_exact(&mut buffer)?;


    Ok(u32::from_le_bytes(buffer))
}
  

fn read_varint(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
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
    let mut bytes = [0u8; 32];

    transaction_bytes.read_exact(&mut bytes)?;

    Ok(Txid::from_bytes(bytes))
}



fn read_script_size(
    transaction_bytes: &mut &[u8]
) -> Result<String, Error> {
    Ok(read_varint(transaction_bytes)? as usize)
}

fn read_version_byte(
    transaction_bytes: &mut &[u8]
) -> Result<u32, Error> {
    read_u32(transaction_bytes)
}

// Bitcoin uses little-endian encoding for most of its numeric fields, meaning the least significant byte comes first.

fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {


}


pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    

}