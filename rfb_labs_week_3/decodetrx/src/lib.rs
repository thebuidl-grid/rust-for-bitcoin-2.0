//! Raw Bitcoin transaction decoder.
//!
//! Reads a raw transaction hex string and produces a JSON description of it.
//! Handles both legacy and SegWit (BIP144) serialization.

use sha2::{Digest, Sha256};
use std::io::{Error, ErrorKind, Read};
use transaction::{Amount, Input, Output, Transaction, Txid};

pub mod transaction;

/// Reads the 4-byte little-endian version straight off a raw transaction hex
/// string. Convenience helper — the decoder itself uses [`read_version_byte`],
/// which reads from the same cursor as every other field.
///
/// Returns 0 if the hex is malformed or shorter than 4 bytes.
pub fn read_version(transaction_hex: &str) -> u32 {
    let Ok(bytes) = hex::decode(transaction_hex.trim()) else {
        return 0;
    };
    if bytes.len() < 4 {
        return 0;
    }
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

/// Reads the next 8 bytes as a little-endian u64.
///
/// # Panics
/// If fewer than 8 bytes remain. Callers must check the length first — see
/// [`read_amount`], which does exactly that.
fn read_u64(transaction_bytes: &mut &[u8]) -> u64 {
    let mut buffer = [0u8; 8];
    transaction_bytes
        .read_exact(&mut buffer)
        .expect("read_u64 called with fewer than 8 bytes remaining");
    u64::from_le_bytes(buffer)
}

/// Reads an 8-byte little-endian satoshi amount.
fn read_amount(transaction_bytes: &mut &[u8]) -> Result<Amount, Error> {
    if transaction_bytes.len() < 8 {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "not enough bytes remaining for an 8-byte amount",
        ));
    }
    Ok(Amount::from_sat(read_u64(transaction_bytes)))
}

/// Reads the next 4 bytes as a little-endian u32.
fn read_u32(bytes_slice: &mut &[u8]) -> Result<u32, Error> {
    let mut buffer = [0u8; 4];
    bytes_slice.read_exact(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

/// Reads a CompactSize (VarInt).
///
/// This is the fundamental building block of a Bitcoin parser: CompactSize
/// encodes the input count, output count, every script length, and every
/// witness element count.
///
/// | First byte | Total width | Value read from       |
/// |------------|-------------|-----------------------|
/// | `0x00..fc` | 1 byte      | the first byte itself |
/// | `0xfd`     | 3 bytes     | next 2 bytes, LE      |
/// | `0xfe`     | 5 bytes     | next 4 bytes, LE      |
/// | `0xff`     | 9 bytes     | next 8 bytes, LE      |
fn read_compact_size(transaction_bytes: &mut &[u8]) -> Result<u64, Error> {
    let mut marker = [0u8; 1];
    transaction_bytes.read_exact(&mut marker)?;

    match marker[0] {
        0x00..=0xfc => Ok(marker[0] as u64),
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

/// Reads a 32-byte transaction ID in wire (internal) byte order.
fn read_txid(transaction_bytes: &mut &[u8]) -> Result<Txid, Error> {
    let mut buffer = [0u8; 32];
    transaction_bytes.read_exact(&mut buffer)?;
    Ok(Txid::from_bytes(buffer))
}

/// Reads a CompactSize-prefixed script and returns it as a hex string.
///
/// Used for scriptSig, scriptPubKey, and each witness stack item.
fn read_script_size(transaction_bytes: &mut &[u8]) -> Result<String, Error> {
    let length = read_compact_size(transaction_bytes)? as usize;

    // Check before allocating: a corrupt length field could otherwise ask for
    // gigabytes on a 200-byte transaction.
    if transaction_bytes.len() < length {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            format!(
                "script claims {} bytes but only {} remain",
                length,
                transaction_bytes.len()
            ),
        ));
    }

    let mut buffer = vec![0u8; length];
    transaction_bytes.read_exact(&mut buffer)?;
    Ok(hex::encode(buffer))
}

/// Reads the 4-byte transaction version field.
// Bitcoin uses little-endian encoding for most of its numeric fields, meaning the least significant byte comes first.
fn read_version_byte(transaction_bytes: &mut &[u8]) -> Result<u32, Error> {
    read_u32(transaction_bytes)
}

/// Computes a txid: SHA256 applied twice over the given serialization.
///
/// Pass the *legacy* serialization (no marker, flag, or witness) to get the
/// txid; pass the full SegWit serialization to get the wtxid.
fn hash_row_transaction(row_transaction_bytes: &[u8]) -> Result<Txid, Error> {
    let first = Sha256::digest(row_transaction_bytes);
    let second = Sha256::digest(first);

    let mut buffer = [0u8; 32];
    buffer.copy_from_slice(&second);
    Ok(Txid::from_bytes(buffer))
}

pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let raw = hex::decode(transaction_hex.trim())?;
    let mut bytes = raw.as_slice();

    let version = read_version_byte(&mut bytes)?;

    // BIP144: a 0x00 where the input count belongs is the SegWit marker, since
    // a real transaction can never have zero inputs. The flag must then be 0x01.
    let is_segwit = bytes.first() == Some(&0x00);
    if is_segwit {
        if bytes.len() < 2 {
            return Err("truncated after SegWit marker".into());
        }
        if bytes[1] != 0x01 {
            return Err(format!("expected SegWit flag 0x01, found {:#04x}", bytes[1]).into());
        }
        bytes = &bytes[2..];
    }

    // Remember where the inputs start so the txid can be computed over the
    // legacy serialization later.
    let inputs_start = raw.len() - bytes.len();

    let input_count = read_compact_size(&mut bytes)?;
    let mut inputs = Vec::with_capacity(input_count.min(1024) as usize);
    for _ in 0..input_count {
        inputs.push(Input {
            txid: read_txid(&mut bytes)?,
            output_index: read_u32(&mut bytes)?,
            script_sig: read_script_size(&mut bytes)?,
            sequence: read_u32(&mut bytes)?,
            witness: Vec::new(),
        });
    }

    let output_count = read_compact_size(&mut bytes)?;
    let mut outputs = Vec::with_capacity(output_count.min(1024) as usize);
    for _ in 0..output_count {
        outputs.push(Output {
            amount: read_amount(&mut bytes)?,
            script_pubkey: read_script_size(&mut bytes)?,
        });
    }

    // Everything from inputs_start to here is what the legacy serialization
    // keeps; the witness block starts at this offset.
    let outputs_end = raw.len() - bytes.len();

    if is_segwit {
        // Each input carries its own witness stack, in input order.
        for input in inputs.iter_mut() {
            let item_count = read_compact_size(&mut bytes)?;
            let mut witness = Vec::with_capacity(item_count.min(1024) as usize);
            for _ in 0..item_count {
                witness.push(read_script_size(&mut bytes)?);
            }
            input.witness = witness;
        }
    }

    let lock_time = read_u32(&mut bytes)?;

    if !bytes.is_empty() {
        return Err(format!(
            "{} trailing byte(s) after lock_time — not a valid transaction",
            bytes.len()
        )
        .into());
    }

    // The txid is always the hash of the legacy serialization, even for SegWit
    // transactions: version ++ inputs ++ outputs ++ lock_time, with the marker,
    // flag, and witness stripped out. Hashing the full SegWit bytes would give
    // the wtxid instead.
    let legacy_serialization = if is_segwit {
        let mut legacy = Vec::with_capacity(8 + outputs_end - inputs_start);
        legacy.extend_from_slice(&raw[..4]);
        legacy.extend_from_slice(&raw[inputs_start..outputs_end]);
        legacy.extend_from_slice(&raw[raw.len() - 4..]);
        legacy
    } else {
        raw.clone()
    };

    let transaction = Transaction {
        transaction_id: hash_row_transaction(&legacy_serialization)?,
        version,
        inputs,
        outputs,
        lock_time,
    };

    Ok(serde_json::to_string_pretty(&transaction)?)
}
