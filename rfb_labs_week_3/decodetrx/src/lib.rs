use std::io::{Error, Read};
use sha2::{Digest, Sha256}; // https://docs.rs/sha2/latest/sha2/
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

// #[allow(unused_variables)]
// fn read_version(transaction_hex: &str) -> u32 {
//  let transaction_bytes = hex::decode(transaction_hex).map_err(|e| format!("Hex decode error: {}", e))?;
//  let mut bytes_slice = transaction_bytes.as_slice();
//  read_u32(&mut bytes_slice);
// }

fn read_u64(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let mut buffer = [0; 8];
    transaction_bytes.read(&mut buffer)?;
    Ok(u64::from_le_bytes(buffer))
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    let mut buffer = [0; 8];
    transaction_bytes.read(&mut buffer)?;
    Ok(Amount::from_sat(u64::from_le_bytes(buffer)))
}

fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    let mut buffer = [0; 4];
    bytes_slice.read(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let mut compact_size = [0; 1];
    transaction_bytes.read(&mut compact_size)?;

    match compact_size[0] {
        0..=252 => Ok(compact_size[0] as u64),
        253 => {
            let mut buffer = [0; 2];
            transaction_bytes.read(&mut buffer)?;
            Ok(u16::from_le_bytes(buffer) as u64)
        }
        254 => {
            let mut buffer = [0; 4];
            transaction_bytes.read(&mut buffer)?;
            Ok(u32::from_le_bytes(buffer) as u64)
        }
        255 => {
            let mut buffer = [0; 8];
            transaction_bytes.read(&mut buffer)?;
            Ok(u64::from_le_bytes(buffer))
        }
    }
}

fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    let mut buffer = [0; 32];
    transaction_bytes.read(&mut buffer)?;
    Ok(Txid::from_bytes(buffer))
}

fn read_script(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    let script_size = read_compact_size(transaction_bytes)? as usize;
    let mut buffer = vec![0_u8; script_size];
    transaction_bytes.read(&mut buffer)?;
    Ok(hex::encode(buffer))
}

fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    let (version_bytes, rest) = transaction_bytes.split_at(4);
    *transaction_bytes = rest;
    Ok(u32::from_le_bytes(version_bytes.try_into().unwrap()))
}
// Bitcoin uses little-endian encoding for most of its numeric fields, meaning the least significant byte comes first.

fn hash_raw_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    let mut hasher = Sha256::new();
    hasher.update(&row_transaction_bytes);
    let hash1 = hasher.finalize();

    let mut hasher = Sha256::new();
    hasher.update(hash1);
    let hash2 = hasher.finalize();

    Ok(Txid::from_bytes(hash2.into()))
}

pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let transaction_bytes =
        hex::decode(transaction_hex).map_err(|e| format!("Hex decode error: {}", e))?;
    let mut bytes_slice = transaction_bytes.as_slice();
    let version = read_u32(&mut bytes_slice)?;

    // SegWit transactions insert marker=0x00, flag=0x01 right after the version.
    // A legacy transaction's input count can never be 0x00 (zero inputs is invalid),
    // so peeking at the next byte reliably tells the two apart.
    let is_segwit = bytes_slice.first() == Some(&0x00);
    if is_segwit {
        let mut marker_flag = [0u8; 2];
        bytes_slice.read_exact(&mut marker_flag)?;
    }

    // Offset where the legacy (marker/flag/witness-free) serialization resumes.
    let legacy_start = transaction_bytes.len() - bytes_slice.len();

    let input_count = read_compact_size(&mut bytes_slice)?;
    let mut inputs = vec![];

    for _ in 0..input_count {
        let txid = read_txid(&mut bytes_slice)?;
        let output_index = read_u32(&mut bytes_slice)?;
        let script_sig = read_script(&mut bytes_slice)?;
        let sequence = read_u32(&mut bytes_slice)?;

        inputs.push(Input {
            txid,
            output_index,
            script_sig,
            sequence,
            witness: vec![],
        });
    }

    let output_count = read_compact_size(&mut bytes_slice)?;
    let mut outputs = vec![];

    for _ in 0..output_count {
        let amount = read_amount(&mut bytes_slice)?;
        let script_pubkey = read_script(&mut bytes_slice)?;

        outputs.push(Output {
            amount,
            script_pubkey,
        })
    }

    // Legacy serialization ends here; witness data (if any) comes next, then locktime.
    let legacy_end = transaction_bytes.len() - bytes_slice.len();

    if is_segwit {
        for input in inputs.iter_mut() {
            let item_count = read_compact_size(&mut bytes_slice)?;
            let mut witness = Vec::with_capacity(item_count as usize);
            for _ in 0..item_count {
                witness.push(read_script(&mut bytes_slice)?);
            }
            input.witness = witness;
        }
    }

    let lock_time = read_u32(&mut bytes_slice)?;

    // The TXID always hashes the legacy serialization (no marker/flag/witness) -
    // hashing the raw SegWit bytes as-is would give the wtxid instead.
    let mut legacy_bytes = Vec::with_capacity(8 + (legacy_end - legacy_start));
    legacy_bytes.extend_from_slice(&transaction_bytes[0..4]); // version
    legacy_bytes.extend_from_slice(&transaction_bytes[legacy_start..legacy_end]); // inputs + outputs
    legacy_bytes.extend_from_slice(&transaction_bytes[transaction_bytes.len() - 4..]); // locktime
    let transaction_id = hash_raw_transaction(&legacy_bytes)?;

    let transaction = Transaction {
        version,
        inputs,
        outputs,
        lock_time,
        transaction_id,
    };

    Ok(serde_json::to_string_pretty(&transaction)?)
}
