use sha2::{Digest, Sha256};
use std::io::Error;
use transaction::{Amount, Input, Output, Transaction, Txid};
mod transaction;

// Pulls `n` bytes off the front of `bytes`, advancing the slice past them.
// Every read_* function below is built on this one bounds-checked step.
fn take<'a>(bytes: &mut &'a [u8], n: usize) -> Result<&'a [u8], Error> {
    if bytes.len() < n {
        return Err(Error::new(
            std::io::ErrorKind::UnexpectedEof,
            format!("expected {n} more bytes, only {} left", bytes.len()),
        ));
    }
    let (chunk, rest) = bytes.split_at(n);
    *bytes = rest;
    Ok(chunk)
}

// Standalone quick-read of just the version field straight from the hex
// string, without going through the byte-cursor machinery below.
fn read_version(transaction_hex: &str) -> u32 {
    let bytes = hex::decode(transaction_hex).expect("invalid hex string");
    u32::from_le_bytes(
        bytes[0..4]
            .try_into()
            .expect("transaction too short for a version field"),
    )
}

fn read_u64(transaction_bytes: &mut &[u8]) -> u64 {
    let chunk = take(transaction_bytes, 8).expect("not enough bytes for a u64");
    u64::from_le_bytes(chunk.try_into().unwrap())
}

fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    let satoshis = read_u64(transaction_bytes);
    Ok(Amount::from_sat(satoshis))
}

fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    let chunk = take(bytes_slice, 4)?;
    Ok(u32::from_le_bytes(chunk.try_into().unwrap()))
}

// Bitcoin uses little-endian encoding for most of its numeric fields, meaning the least significant byte comes first.

// This function is a fundamental building block in a Bitcoin parser because CompactSize integers are used throughout the protocol to
// encode the number of transaction inputs, outputs, script lengths, witness element counts, and many other variable-length fields.
fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let prefix = take(transaction_bytes, 1)?[0];
    let value = match prefix {
        0x00..=0xfc => prefix as u64,
        0xfd => {
            let chunk = take(transaction_bytes, 2)?;
            u16::from_le_bytes(chunk.try_into().unwrap()) as u64
        }
        0xfe => {
            let chunk = take(transaction_bytes, 4)?;
            u32::from_le_bytes(chunk.try_into().unwrap()) as u64
        }
        0xff => {
            let chunk = take(transaction_bytes, 8)?;
            u64::from_le_bytes(chunk.try_into().unwrap())
        }
    };
    Ok(value)
}

fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    let chunk = take(transaction_bytes, 32)?;
    let mut array = [0u8; 32];
    array.copy_from_slice(chunk);
    Ok(Txid::from_bytes(array))
}

// Scripts (scriptSig, scriptPubKey, witness items) are all encoded the
// same way on the wire: a CompactSize length, then that many raw bytes.
// Returned as a hex string, which is how they're conventionally displayed.
fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    let length = read_compact_size(transaction_bytes)? as usize;
    let script = take(transaction_bytes, length)?;
    Ok(hex::encode(script))
}

fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    let byte = take(transaction_bytes, 1)?[0];
    Ok(byte as u32)
}

fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    let first_pass = Sha256::digest(row_transaction_bytes);
    let second_pass = Sha256::digest(first_pass);
    let mut array = [0u8; 32];
    array.copy_from_slice(&second_pass);
    Ok(Txid::from_bytes(array))
}

fn encode_compact_size(value: u64) -> Vec<u8> {
    match value {
        0..=0xfc => vec![value as u8],
        0xfd..=0xffff => {
            let mut out = vec![0xfd];
            out.extend_from_slice(&(value as u16).to_le_bytes());
            out
        }
        0x10000..=0xffff_ffff => {
            let mut out = vec![0xfe];
            out.extend_from_slice(&(value as u32).to_le_bytes());
            out
        }
        _ => {
            let mut out = vec![0xff];
            out.extend_from_slice(&value.to_le_bytes());
            out
        }
    }
}

// Rebuilds the *legacy* (witness-stripped) serialization of the
// transaction. A txid is always double-SHA256 of this form, even for a
// SegWit transaction -- witness data is never part of the txid.
fn legacy_serialize(version: u32, inputs: &[Input], outputs: &[Output], lock_time: u32) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&version.to_le_bytes());

    out.extend_from_slice(&encode_compact_size(inputs.len() as u64));
    for input in inputs {
        out.extend_from_slice(input.txid.as_bytes());
        out.extend_from_slice(&input.output_index.to_le_bytes());
        out.extend_from_slice(&encode_compact_size(input.script_sig.len() as u64));
        out.extend_from_slice(&input.script_sig);
        out.extend_from_slice(&input.sequence.to_le_bytes());
    }

    out.extend_from_slice(&encode_compact_size(outputs.len() as u64));
    for output in outputs {
        out.extend_from_slice(&output.amount.to_sat().to_le_bytes());
        out.extend_from_slice(&encode_compact_size(output.script_pubkey.len() as u64));
        out.extend_from_slice(&output.script_pubkey);
    }

    out.extend_from_slice(&lock_time.to_le_bytes());
    out
}

pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let version_preview = read_version(&transaction_hex);

    let full_bytes = hex::decode(transaction_hex.trim())?;
    let mut cursor: &[u8] = &full_bytes;

    let version = read_u32(&mut cursor)?;
    debug_assert_eq!(
        version_preview, version,
        "quick version read disagreed with the cursor-based read"
    );

    // SegWit transactions insert a marker (0x00) and flag (0x01) right
    // after the version. A legacy transaction jumps straight to the
    // input count instead, so peek before consuming anything.
    let is_segwit = cursor.len() >= 2 && cursor[0] == 0x00 && cursor[1] == 0x01;
    if is_segwit {
        read_version_byte(&mut cursor)?; // marker
        read_version_byte(&mut cursor)?; // flag
    }

    let input_count = read_compact_size(&mut cursor)?;
    let mut inputs = Vec::with_capacity(input_count as usize);
    for _ in 0..input_count {
        let txid = read_txid(&mut cursor)?;
        let output_index = read_u32(&mut cursor)?;
        let script_sig = hex::decode(read_script_size(&mut cursor)?)?;
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
        let script_pubkey = hex::decode(read_script_size(&mut cursor)?)?;
        outputs.push(Output {
            amount,
            script_pubkey,
        });
    }

    // Witness data (present only for SegWit inputs) has to be consumed
    // to reach locktime, even though it isn't part of the final struct --
    // it never affects the txid.
    if is_segwit {
        for _ in 0..input_count {
            let item_count = read_compact_size(&mut cursor)?;
            for _ in 0..item_count {
                read_script_size(&mut cursor)?;
            }
        }
    }

    let lock_time = read_u32(&mut cursor)?;

    let legacy_bytes = legacy_serialize(version, &inputs, &outputs, lock_time);
    let transaction_id = hash_row_transaction(&legacy_bytes)?;

    let transaction = Transaction {
        transaction_id,
        version,
        inputs,
        outputs,
        lock_time,
    };

    Ok(serde_json::to_string_pretty(&transaction)?)
}
