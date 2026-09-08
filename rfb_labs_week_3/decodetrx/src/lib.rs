use std::io::Error;
use sha2::{Sha256, Digest};
use transaction::{Amount, Input, Output, Transaction, Txid};

mod transaction;

// === Byte readers
// All readers advance the slice by consuming bytes from the front.

fn read_u8(bytes: &mut &[u8]) -> Result<u8, Error> {
    if bytes.is_empty() {
        return Err(Error::other("unexpected end of data reading u8"));
    }
    let val = bytes[0];
    *bytes = &bytes[1..];
    Ok(val)
}

fn read_u32(bytes: &mut &[u8]) -> Result<u32, Error> {
    if bytes.len() < 4 {
        return Err(Error::other("unexpected end of data reading u32"));
    }
    let val = u32::from_le_bytes(bytes[..4].try_into().unwrap());
    *bytes = &bytes[4..];
    Ok(val)
}

fn read_u64(bytes: &mut &[u8]) -> Result<u64, Error> {
    if bytes.len() < 8 {
        return Err(Error::other("unexpected end of data reading u64"));
    }
    let val = u64::from_le_bytes(bytes[..8].try_into().unwrap());
    *bytes = &bytes[8..];
    Ok(val)
}

fn read_amount(bytes: &mut &[u8]) -> Result<Amount, Error> {
    let sats = read_u64(bytes)?;
    Ok(Amount::from_sat(sats))
}

// CompactSize / VarInt: variable-length integer used throughout Bitcoin's wire format.
//
//   0x00..=0xfc  => value is the byte itself (1 byte total)
//   0xfd         => next 2 bytes are the value, little-endian (3 bytes total)
//   0xfe         => next 4 bytes are the value, little-endian (5 bytes total)
//   0xff         => next 8 bytes are the value, little-endian (9 bytes total)
fn read_compact_size(bytes: &mut &[u8]) -> Result<u64, Error> {
    let first = read_u8(bytes)?;
    match first {
        0x00..=0xfc => Ok(first as u64),
        0xfd => {
            if bytes.len() < 2 {
                return Err(Error::other("unexpected end of data reading compact_size fd"));
            }
            let val = u16::from_le_bytes(bytes[..2].try_into().unwrap()) as u64;
            *bytes = &bytes[2..];
            Ok(val)
        }
        0xfe => {
            if bytes.len() < 4 {
                return Err(Error::other("unexpected end of data reading compact_size fe"));
            }
            let val = u32::from_le_bytes(bytes[..4].try_into().unwrap()) as u64;
            *bytes = &bytes[4..];
            Ok(val)
        }
        // 0xff
        _ => {
            if bytes.len() < 8 {
                return Err(Error::other("unexpected end of data reading compact_size ff"));
            }
            let val = u64::from_le_bytes(bytes[..8].try_into().unwrap());
            *bytes = &bytes[8..];
            Ok(val)
        }
    }
}

// Read exactly n bytes and return them, advancing the slice.
fn read_bytes_n(bytes: &mut &[u8], n: usize) -> Result<Vec<u8>, Error> {
    if bytes.len() < n {
        return Err(Error::other(format!(
            "unexpected end of data: need {n} bytes, have {}",
            bytes.len()
        )));
    }
    let chunk = bytes[..n].to_vec();
    *bytes = &bytes[n..];
    Ok(chunk)
}

// Read a compact-size-prefixed byte vector and return it as a lowercase hex string.
fn read_script(bytes: &mut &[u8]) -> Result<String, Error> {
    let len = read_compact_size(bytes)? as usize;
    let raw = read_bytes_n(bytes, len)?;
    Ok(hex::encode(raw))
}

// Read the 4-byte version field (little-endian u32).
fn read_version_byte(bytes: &mut &[u8]) -> Result<u32, Error> {
    read_u32(bytes)
}

// Read the 32-byte previous-output TXID from an input.
// The bytes arrive in internal byte order (little-endian); we store them as-is
// and reverse only at display time inside Txid::serialize.
fn read_txid(bytes: &mut &[u8]) -> Result<Txid, Error> {
    let raw = read_bytes_n(bytes, 32)?;
    let arr: [u8; 32] = raw.try_into().unwrap();
    Ok(Txid::from_bytes(arr))
}

// Compute the transaction ID by double-SHA256 over the non-witness serialization.
// For SegWit transactions the TXID is computed without marker, flag, and witness fields.
fn hash_transaction(raw_bytes: &[u8]) -> Result<Txid, Error> {
    let first = Sha256::digest(raw_bytes);
    let second = Sha256::digest(first);
    let arr: [u8; 32] = second.into();
    // The TXID is the double-SHA256 result stored as-is;
    // reversal to display format happens inside Txid::serialize.
    Ok(Txid::from_bytes(arr))
}

// Build the non-witness serialization from a raw SegWit transaction so we can hash it.
// Non-witness format: version | inputs | outputs | locktime (no marker/flag/witness).
fn strip_witness(raw: &[u8]) -> Result<Vec<u8>, Error> {
    let mut cursor: &[u8] = raw;
    let mut out: Vec<u8> = Vec::new();

    // version (4 bytes)
    let version_bytes = read_bytes_n(&mut cursor, 4)?;
    out.extend_from_slice(&version_bytes);

    // peek at marker/flag
    if cursor.len() >= 2 && cursor[0] == 0x00 && cursor[1] == 0x01 {
        // skip marker and flag
        cursor = &cursor[2..];
    }

    // input count
    let in_count_start = cursor;
    let in_count = read_compact_size(&mut cursor)?;
    let consumed = in_count_start.len() - cursor.len();
    out.extend_from_slice(&in_count_start[..consumed]);

    for _ in 0..in_count {
        // prev txid (32) + vout (4)
        let header = read_bytes_n(&mut cursor, 36)?;
        out.extend_from_slice(&header);

        // scriptSig (compact-size prefixed)
        let ss_len_start = cursor;
        let ss_len = read_compact_size(&mut cursor)?;
        let ss_len_bytes = ss_len_start.len() - cursor.len();
        out.extend_from_slice(&ss_len_start[..ss_len_bytes]);
        let script_sig = read_bytes_n(&mut cursor, ss_len as usize)?;
        out.extend_from_slice(&script_sig);

        // sequence (4)
        let seq = read_bytes_n(&mut cursor, 4)?;
        out.extend_from_slice(&seq);
    }

    // output count
    let out_count_start = cursor;
    let out_count = read_compact_size(&mut cursor)?;
    let consumed = out_count_start.len() - cursor.len();
    out.extend_from_slice(&out_count_start[..consumed]);

    for _ in 0..out_count {
        // value (8)
        let val = read_bytes_n(&mut cursor, 8)?;
        out.extend_from_slice(&val);

        // scriptPubKey (compact-size prefixed)
        let pk_len_start = cursor;
        let pk_len = read_compact_size(&mut cursor)?;
        let pk_len_bytes = pk_len_start.len() - cursor.len();
        out.extend_from_slice(&pk_len_start[..pk_len_bytes]);
        let script_pubkey = read_bytes_n(&mut cursor, pk_len as usize)?;
        out.extend_from_slice(&script_pubkey);
    }

    // skip witness fields (one stack per input)
    for _ in 0..in_count {
        let item_count = read_compact_size(&mut cursor)?;
        for _ in 0..item_count {
            let item_len = read_compact_size(&mut cursor)? as usize;
            read_bytes_n(&mut cursor, item_len)?;
        }
    }

    // locktime (4)
    out.extend_from_slice(&cursor[..4]);

    Ok(out)
}

// === decode_transaction
//
// Accepts a raw hex-encoded Bitcoin transaction (legacy or SegWit),
// decodes every field, and returns a pretty-printed JSON string.
pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    let raw = hex::decode(transaction_hex.trim())?;
    let mut cursor: &[u8] = &raw;

    // version
    let version = read_version_byte(&mut cursor)?;

    // detect SegWit: marker 0x00 followed by flag 0x01
    let segwit = cursor.len() >= 2 && cursor[0] == 0x00 && cursor[1] == 0x01;
    if segwit {
        cursor = &cursor[2..]; // consume marker + flag
    }

    // inputs
    let in_count = read_compact_size(&mut cursor)? as usize;
    let mut inputs: Vec<Input> = Vec::with_capacity(in_count);

    for _ in 0..in_count {
        let txid = read_txid(&mut cursor)?;
        let output_index = read_u32(&mut cursor)?;
        let script_sig = read_script(&mut cursor)?;
        let sequence = read_u32(&mut cursor)?;

        inputs.push(Input {
            txid,
            output_index,
            script_sig,
            sequence,
            witness: Vec::new(), // filled in below for SegWit
        });
    }

    // outputs
    let out_count = read_compact_size(&mut cursor)? as usize;
    let mut outputs: Vec<Output> = Vec::with_capacity(out_count);

    for _ in 0..out_count {
        let amount = read_amount(&mut cursor)?;
        let script_pubkey = read_script(&mut cursor)?;

        outputs.push(Output { amount, script_pubkey });
    }

    // witness stacks (one per input, only present in SegWit transactions)
    if segwit {
        for input in inputs.iter_mut() {
            let item_count = read_compact_size(&mut cursor)? as usize;
            let mut witness: Vec<String> = Vec::with_capacity(item_count);

            for _ in 0..item_count {
                let item_len = read_compact_size(&mut cursor)? as usize;
                let item_bytes = read_bytes_n(&mut cursor, item_len)?;
                witness.push(hex::encode(item_bytes));
            }
            input.witness = witness;
        }
    }

    // locktime
    let lock_time = read_u32(&mut cursor)?;

    // compute TXID
    // SegWit TXIDs are hashed over the non-witness serialization
    let txid_bytes = if segwit {
        strip_witness(&raw)?
    } else {
        raw.clone()
    };
    let transaction_id = hash_transaction(&txid_bytes)?;

    let tx = Transaction {
        transaction_id,
        version,
        inputs,
        outputs,
        lock_time,
    };

    Ok(serde_json::to_string_pretty(&tx)?)
}
