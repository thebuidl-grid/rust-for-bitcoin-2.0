//! Parses a raw Bitcoin transaction into a JSON object.
//!
//! Where `decodetrx` builds typed structs and lets serde derive the JSON, this
//! crate walks the bytes with a [`Cursor`] and assembles the JSON by hand — the
//! same transaction layout viewed through a different lens.
//!
//! ```text
//! ┌──────────────────────────────┐
//! │ Version          4 bytes     │
//! ├──────────────────────────────┤
//! │ Marker           1 byte      │  SegWit only
//! │ Flag             1 byte      │  SegWit only
//! ├──────────────────────────────┤
//! │ Input count      VarInt      │
//! │ Inputs           Variable    │
//! ├──────────────────────────────┤
//! │ Output count     VarInt      │
//! │ Outputs          Variable    │
//! ├──────────────────────────────┤
//! │ Witness          Variable    │  SegWit only
//! ├──────────────────────────────┤
//! │ Locktime         4 bytes     │
//! └──────────────────────────────┘
//! ```

use byteorder::{LittleEndian, ReadBytesExt};
use serde_json::{Value, json};
use std::io::{Cursor, Read};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// read_u64: read the next 8 bytes
// read_u32: read the next 4 bytes
// read_u16: read the next 2 bytes
// read_u8:  read the next 1 byte

/// Reads a CompactSize / VarInt.
///
/// This function is a fundamental building block in a Bitcoin parser because
/// CompactSize integers are used throughout the protocol to encode the number
/// of transaction inputs, outputs, script lengths, witness element counts, and
/// many other variable-length fields.
fn read_varint(r: &mut Cursor<Vec<u8>>) -> Result<u64> {
    let n = r.read_u8()?;
    Ok(match n {
        0x00..=0xfc => n as u64,
        0xfd => r.read_u16::<LittleEndian>()? as u64,
        0xfe => r.read_u32::<LittleEndian>()? as u64,
        _ => r.read_u64::<LittleEndian>()?,
    })
}

fn read_bytes(r: &mut Cursor<Vec<u8>>, n: usize) -> Result<Vec<u8>> {
    // Guard before allocating — a corrupt length field could otherwise ask for
    // gigabytes on a 200-byte transaction.
    let remaining = r.get_ref().len() - (r.position() as usize).min(r.get_ref().len());
    if remaining < n {
        return Err(format!("field claims {} bytes but only {} remain", n, remaining).into());
    }

    let mut b = vec![0; n]; // creates a Vec<u8> with n elements, where each element is one byte (u8).
    r.read_exact(&mut b)?; // read exactly n bytes from the last position of the cursor,
    // The cursor moves forward by n bytes.
    Ok(b)
}

fn hex(v: &[u8]) -> String {
    v.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Parses a raw transaction hex string into a JSON object.
pub fn parse_transaction(raw_hex: &str) -> Result<Value> {
    let bytes = hex::decode(raw_hex.trim())?;
    let mut r = Cursor::new(bytes);

    // When we read 4 bytes, the cursor automatically moves forward.
    // read_u32: Read the next 4 bytes and interpret them as an unsigned 32-bit integer.
    // LittleEndian: the least significant byte comes first.
    // We have 02 00 00 00 — little-endian turns it into 0x00000002.
    let version = r.read_u32::<LittleEndian>()?;

    // A 0x00 where the input count belongs is the SegWit marker, since a real
    // transaction can never have zero inputs. The flag must then be 0x01.
    let position = r.position();
    let segwit = r.read_u8()? == 0x00;
    if segwit {
        let flag = r.read_u8()?;
        if flag != 0x01 {
            return Err(format!("expected SegWit flag 0x01, found {:#04x}", flag).into());
        }
    } else {
        // Not SegWit — rewind, that byte was the input count.
        r.set_position(position);
    }

    let in_count = read_varint(&mut r)?;
    let mut inputs = Vec::new();
    for _ in 0..in_count {
        // Each input contains a 32-byte previous transaction hash (TXID),
        // stored on the wire in internal byte order and displayed reversed.
        let mut prev = read_bytes(&mut r, 32)?;
        prev.reverse();
        let vout = r.read_u32::<LittleEndian>()?;
        let slen = read_varint(&mut r)? as usize;
        let script = read_bytes(&mut r, slen)?;
        let seq = r.read_u32::<LittleEndian>()?;

        inputs.push(json!({
            "prev_txid": hex(&prev),
            "vout": vout,
            "script_sig": hex(&script),
            "sequence": format!("{:08x}", seq),
        }));
    }

    let out_count = read_varint(&mut r)?;
    let mut outputs = Vec::new();
    for _ in 0..out_count {
        // Read the next 8 bytes and interpret them as a little-endian u64.
        let value = r.read_u64::<LittleEndian>()?;
        let slen = read_varint(&mut r)? as usize;
        let script = read_bytes(&mut r, slen)?;

        outputs.push(json!({
            "value_sats": value,
            "value_btc": value as f64 / 100_000_000.0,
            "script_pubkey": hex(&script),
        }));
    }

    // Each input has its own witness field, in input order.
    if segwit {
        for input in inputs.iter_mut() {
            let items = read_varint(&mut r)?;
            let mut witness = Vec::new();
            for _ in 0..items {
                let len = read_varint(&mut r)? as usize;
                witness.push(hex(&read_bytes(&mut r, len)?));
            }
            input["witness"] = json!(witness);
        }
    }

    let locktime = r.read_u32::<LittleEndian>()?;

    let consumed = r.position() as usize;
    let total = r.get_ref().len();
    if consumed != total {
        return Err(format!("{} trailing byte(s) after locktime", total - consumed).into());
    }

    Ok(json!({
        "version": version,
        "segwit": segwit,
        "inputs": inputs,
        "outputs": outputs,
        "locktime": locktime,
    }))
}
