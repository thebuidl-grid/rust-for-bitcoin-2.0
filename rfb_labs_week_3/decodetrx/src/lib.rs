use std::io::{Read, Error};
use sha2::{Sha256, Digest};
use transaction::{Amount, Input, Output, Transaction, Txid};

mod transaction;

fn read_version(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    let mut buffer = [0u8; 4];
    transaction_bytes.read_exact(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    let mut buffer = [0u8; 4];
    bytes_slice.read_exact(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

fn read_u64(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let mut buffer = [0u8; 8];
    transaction_bytes.read_exact(&mut buffer)?;
    Ok(u64::from_le_bytes(buffer))
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    let satoshis = read_u64(transaction_bytes)?;
    Ok(Amount::from_sat(satoshis))
}

fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let mut first_byte = [0u8; 1];
    transaction_bytes.read_exact(&mut first_byte)?;
    
    match first_byte[0] {
        0..=252 => Ok(first_byte[0] as u64),
        253 => {
            let mut buffer = [0u8; 2];
            transaction_bytes.read_exact(&mut buffer)?;
            Ok(u16::from_le_bytes(buffer) as u64)
        }
        254 => {
            let mut buffer = [0u8; 4];
            transaction_bytes.read_exact(&mut buffer)?;
            Ok(u32::from_le_bytes(buffer) as u64)
        }
        255 => {
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

fn read_script(transaction_bytes: &mut &[u8]) -> Result<Vec<u8>, Error> {
    let script_size = read_compact_size(transaction_bytes)? as usize;
    let mut script = vec![0u8; script_size];
    transaction_bytes.read_exact(&mut script)?;
    Ok(script)
}

fn hash_raw_transaction(raw_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    let mut hasher = Sha256::new();
    hasher.update(raw_transaction_bytes);
    let first_hash = hasher.finalize();
    
    let mut hasher = Sha256::new();
    hasher.update(&first_hash);
    let second_hash = hasher.finalize();
    
    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&second_hash);
    
    Ok(Txid::from_bytes_reversed(hash_array))
}

fn parse_inputs(transaction_bytes: &mut &[u8], input_count: u64) -> Result<Vec<Input>, Error> {
    let mut inputs = Vec::new();
    
    for _ in 0..input_count {
        let txid = read_txid(transaction_bytes)?;
        let output_index = read_u32(transaction_bytes)?;
        let script_sig = read_script(transaction_bytes)?;
        let sequence = read_u32(transaction_bytes)?;
        
        inputs.push(Input {
            txid,
            output_index,
            script_sig,
            sequence,
        });
    }
    
    Ok(inputs)
}

fn parse_outputs(transaction_bytes: &mut &[u8], output_count: u64) -> Result<Vec<Output>, Error> {
    let mut outputs = Vec::new();
    
    for _ in 0..output_count {
        let amount = read_amount(transaction_bytes)?;
        let script_pubkey = read_script(transaction_bytes)?;
        
        outputs.push(Output {
            amount,
            script_pubkey,
        });
    }
    
    Ok(outputs)
}

pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = hex::decode(&transaction_hex)?;
    let mut bytes_slice = bytes.as_slice();
    
    let raw_bytes = bytes.clone();
    
    let version = read_version(&mut bytes_slice)?;
    let input_count = read_compact_size(&mut bytes_slice)?;
    let inputs = parse_inputs(&mut bytes_slice, input_count)?;
    let output_count = read_compact_size(&mut bytes_slice)?;
    let outputs = parse_outputs(&mut bytes_slice, output_count)?;
    let lock_time = read_u32(&mut bytes_slice)?;
    
    let transaction_id = hash_raw_transaction(&raw_bytes)?;
    
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
