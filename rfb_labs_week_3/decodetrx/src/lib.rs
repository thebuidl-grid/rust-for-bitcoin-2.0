use std::io::{Error, ErrorKind};
use sha2::{Sha256, Digest};
use transaction::{Amount, Input, Output, Transaction, Txid};

mod transaction;

// ──────────────────────────────────────────────────────────────
// Low-level byte readers
// All functions take `&mut &[u8]` – a mutable reference to a
// byte-slice reference.  Reading advances the slice in-place so
// the next call picks up right where the last one left off.
// ──────────────────────────────────────────────────────────────

/// Read 4 bytes and decode as a little-endian u32.
/// Used for: version, output_index, sequence, lock_time.
fn read_u32(bytes: &mut &[u8]) -> Result<u32, Error> {
    // We need exactly 4 bytes
    if bytes.len() < 4 {
        return Err(Error::new(ErrorKind::UnexpectedEof, "not enough bytes for u32"));
    }
    // Copy the first 4 bytes into a fixed-size array
    let arr: [u8; 4] = bytes[..4].try_into().unwrap();
    // Advance the slice past those 4 bytes
    *bytes = &bytes[4..];
    // Bitcoin uses little-endian: least significant byte first
    Ok(u32::from_le_bytes(arr))
}

/// Read 8 bytes and decode as a little-endian u64.
/// Used for: output value (satoshis).
fn read_u64(bytes: &mut &[u8]) -> u64 {
    // Panic on short input — consistent with the trxparse reference style
    let arr: [u8; 8] = bytes[..8].try_into().expect("not enough bytes for u64");
    *bytes = &bytes[8..];
    u64::from_le_bytes(arr)
}

/// Read the version field (first 4 bytes of any Bitcoin transaction).
/// Takes the original hex string, decodes it, and returns the u32 version.
/// This is a convenience wrapper kept as a teaching reference.
#[allow(dead_code)]
fn read_version(transaction_hex: &str) -> u32 {
    let bytes = hex::decode(transaction_hex).expect("invalid hex");
    // Version is always the first 4 bytes, little-endian
    u32::from_le_bytes(bytes[..4].try_into().unwrap())
}

/// Read an output amount (8 bytes, little-endian satoshis) and wrap it in Amount.
fn read_amount(bytes: &mut &[u8]) -> Result<Amount, Error> {
    let sats = read_u64(bytes);
    Ok(Amount::from_sat(sats))
}

/// Read a CompactSize (VarInt) integer.
///
/// Bitcoin uses this variable-length encoding for counts and lengths:
///   0x00–0xfc  → value is the byte itself            (1 byte total)
///   0xfd       → next 2 bytes are the value LE        (3 bytes total)
///   0xfe       → next 4 bytes are the value LE        (5 bytes total)
///   0xff       → next 8 bytes are the value LE        (9 bytes total)
fn read_compact_size(bytes: &mut &[u8]) -> Result<u64, Error> {
    if bytes.is_empty() {
        return Err(Error::new(ErrorKind::UnexpectedEof, "empty buffer reading compact size"));
    }
    // Read the first (discriminant) byte
    let first = bytes[0];
    *bytes = &bytes[1..];

    match first {
        // Small value – the byte IS the number
        0x00..=0xfc => Ok(first as u64),

        // 0xfd: the real value is in the next 2 bytes (little-endian u16)
        0xfd => {
            let arr: [u8; 2] = bytes[..2].try_into()
                .map_err(|_| Error::new(ErrorKind::UnexpectedEof, "not enough bytes for u16 varint"))?;
            *bytes = &bytes[2..];
            Ok(u16::from_le_bytes(arr) as u64)
        }

        // 0xfe: the real value is in the next 4 bytes (little-endian u32)
        0xfe => {
            let arr: [u8; 4] = bytes[..4].try_into()
                .map_err(|_| Error::new(ErrorKind::UnexpectedEof, "not enough bytes for u32 varint"))?;
            *bytes = &bytes[4..];
            Ok(u32::from_le_bytes(arr) as u64)
        }

        // 0xff: the real value is in the next 8 bytes (little-endian u64)
        _ => {
            let arr: [u8; 8] = bytes[..8].try_into()
                .map_err(|_| Error::new(ErrorKind::UnexpectedEof, "not enough bytes for u64 varint"))?;
            *bytes = &bytes[8..];
            Ok(u64::from_le_bytes(arr))
        }
    }
}

/// Read exactly `n` raw bytes and return them as a Vec<u8>.
fn read_bytes_n(bytes: &mut &[u8], n: usize) -> Result<Vec<u8>, Error> {
    if bytes.len() < n {
        return Err(Error::new(ErrorKind::UnexpectedEof, format!("need {n} bytes, have {}", bytes.len())));
    }
    let chunk = bytes[..n].to_vec();
    *bytes = &bytes[n..];
    Ok(chunk)
}

/// Read a 32-byte transaction ID.
/// Bitcoin stores TXIDs in little-endian order in the raw bytes.
/// We keep them as-is here; the Serialize impl on Txid handles reversing for display.
fn read_txid(bytes: &mut &[u8]) -> Result<Txid, Error> {
    let raw = read_bytes_n(bytes, 32)?;
    // Convert Vec<u8> into a fixed [u8; 32] array
    let arr: [u8; 32] = raw.try_into()
        .map_err(|_| Error::new(ErrorKind::Other, "txid must be 32 bytes"))?;
    Ok(Txid::from_bytes(arr))
}

/// Read a script field: first read its CompactSize length, then read that many
/// bytes and hex-encode them into a String.
/// Used for both scriptSig (inputs) and scriptPubKey (outputs).
fn read_script(bytes: &mut &[u8]) -> Result<String, Error> {
    let len = read_compact_size(bytes)? as usize;
    let script_bytes = read_bytes_n(bytes, len)?;
    // Return the script as a lowercase hex string (matches block explorer format)
    Ok(hex::encode(script_bytes))
}

/// Read the 4-byte version and return it as u32.
/// Alias used inside decode_transaction for clarity.
fn read_version_byte(bytes: &mut &[u8]) -> Result<u32, Error> {
    read_u32(bytes)
}

// ──────────────────────────────────────────────────────────────
// TXID computation
// ──────────────────────────────────────────────────────────────

/// Compute the Bitcoin TXID for a raw transaction.
///
/// Bitcoin's TXID = SHA256(SHA256(raw_tx_bytes))
/// The result is a 32-byte array.  Bitcoin traditionally displays it
/// in reversed byte order, which is handled by Txid's Serialize impl.
///
/// For SegWit transactions the TXID is computed over the **non-witness**
/// serialization (stripping marker, flag, and witness fields).  This
/// function receives the full raw bytes and strips those fields if present.
fn hash_raw_transaction(raw_bytes: &[u8]) -> Result<Txid, Error> {
    // Determine whether this is a SegWit transaction.
    // Byte 4 = marker (0x00) and byte 5 = flag (0x01) indicate SegWit.
    let is_segwit = raw_bytes.len() > 5 && raw_bytes[4] == 0x00 && raw_bytes[5] == 0x01;

    let hash_input: Vec<u8> = if is_segwit {
        // Build the legacy (non-witness) serialization for TXID hashing:
        //   version (4) + inputs + outputs + locktime (4)
        // i.e. the full bytes minus the marker/flag bytes and witness data.
        build_non_witness_bytes(raw_bytes)?
    } else {
        raw_bytes.to_vec()
    };

    // First SHA-256 pass
    let first_hash = Sha256::digest(&hash_input);
    // Second SHA-256 pass (double-SHA256)
    let second_hash = Sha256::digest(&first_hash);

    let arr: [u8; 32] = second_hash.into();
    Ok(Txid::from_bytes(arr))
}

/// Strip SegWit marker, flag, and witness fields from raw bytes so we get the
/// legacy serialization needed for TXID calculation.
fn build_non_witness_bytes(raw: &[u8]) -> Result<Vec<u8>, Error> {
    let mut bytes: &[u8] = raw;
    let mut out: Vec<u8> = Vec::new();

    // version: 4 bytes
    let version_bytes = read_bytes_n(&mut bytes, 4)?;
    out.extend_from_slice(&version_bytes);

    // skip marker (0x00) and flag (0x01) – 2 bytes
    let _marker = read_bytes_n(&mut bytes, 2)?;

    // input count (CompactSize)
    let in_count_byte_start = bytes;
    let in_count = read_compact_size(&mut bytes)?;
    // Write the CompactSize bytes for input count
    let consumed = in_count_byte_start.len() - bytes.len();
    out.extend_from_slice(&in_count_byte_start[..consumed]);

    // inputs
    for _ in 0..in_count {
        // prev txid: 32 bytes
        out.extend_from_slice(&read_bytes_n(&mut bytes, 32)?);
        // vout: 4 bytes
        out.extend_from_slice(&read_bytes_n(&mut bytes, 4)?);
        // scriptSig (with its CompactSize length prefix)
        let script_len_start = bytes;
        let script_len = read_compact_size(&mut bytes)?;
        let varint_consumed = script_len_start.len() - bytes.len();
        out.extend_from_slice(&script_len_start[..varint_consumed]);
        out.extend_from_slice(&read_bytes_n(&mut bytes, script_len as usize)?);
        // sequence: 4 bytes
        out.extend_from_slice(&read_bytes_n(&mut bytes, 4)?);
    }

    // output count (CompactSize)
    let out_count_start = bytes;
    let out_count = read_compact_size(&mut bytes)?;
    let consumed = out_count_start.len() - bytes.len();
    out.extend_from_slice(&out_count_start[..consumed]);

    // outputs
    for _ in 0..out_count {
        // value: 8 bytes
        out.extend_from_slice(&read_bytes_n(&mut bytes, 8)?);
        // scriptPubKey (with its CompactSize length prefix)
        let spk_len_start = bytes;
        let spk_len = read_compact_size(&mut bytes)?;
        let varint_consumed = spk_len_start.len() - bytes.len();
        out.extend_from_slice(&spk_len_start[..varint_consumed]);
        out.extend_from_slice(&read_bytes_n(&mut bytes, spk_len as usize)?);
    }

    // skip witness data for each input (not included in TXID hash)
    for _ in 0..in_count {
        let item_count = read_compact_size(&mut bytes)?;
        for _ in 0..item_count {
            let item_len = read_compact_size(&mut bytes)? as usize;
            read_bytes_n(&mut bytes, item_len)?;
        }
    }

    // locktime: 4 bytes
    out.extend_from_slice(&read_bytes_n(&mut bytes, 4)?);

    Ok(out)
}

// ──────────────────────────────────────────────────────────────
// Public entry point
// ──────────────────────────────────────────────────────────────

/// Decode a raw Bitcoin transaction hex string into a JSON string.
///
/// Handles both legacy and SegWit (BIP141) transactions.
/// Returns pretty-printed JSON with:
///   - transaction_id (reversed hex)
///   - version
///   - inputs  (txid, output_index, script_sig, sequence)
///   - outputs (amount in BTC, script_pubkey hex)
///   - lock_time
pub fn decode_transaction(transaction_hex: String) -> Result<String, Box<dyn std::error::Error>> {
    // ── 1. Hex → bytes ────────────────────────────────────────
    let raw_bytes = hex::decode(&transaction_hex)?;

    // ── 2. Compute TXID (double-SHA256, reversed for display) ─
    let transaction_id = hash_raw_transaction(&raw_bytes)?;

    // ── 3. Parse fields ───────────────────────────────────────
    let mut bytes: &[u8] = &raw_bytes;

    // Version (4 bytes, little-endian)
    let version = read_version_byte(&mut bytes)?;

    // SegWit detection: marker=0x00, flag=0x01
    let is_segwit = bytes.len() >= 2 && bytes[0] == 0x00 && bytes[1] == 0x01;
    if is_segwit {
        // Consume the marker and flag bytes
        bytes = &bytes[2..];
    }

    // ── 4. Inputs ─────────────────────────────────────────────
    let input_count = read_compact_size(&mut bytes)?;
    let mut inputs: Vec<Input> = Vec::with_capacity(input_count as usize);

    for _ in 0..input_count {
        // Previous TXID: 32 bytes (stored little-endian, reversed on display by Txid::serialize)
        let txid = read_txid(&mut bytes)?;
        // Which output of the previous transaction we spend (4 bytes LE)
        let output_index = read_u32(&mut bytes)?;
        // scriptSig: CompactSize length + script bytes → hex string
        let script_sig = read_script(&mut bytes)?;
        // Sequence: 4 bytes LE
        let sequence = read_u32(&mut bytes)?;

        inputs.push(Input { txid, output_index, script_sig, sequence });
    }

    // ── 5. Outputs ────────────────────────────────────────────
    let output_count = read_compact_size(&mut bytes)?;
    let mut outputs: Vec<Output> = Vec::with_capacity(output_count as usize);

    for _ in 0..output_count {
        // Amount: 8 bytes LE satoshis (serialized as BTC via as_btc)
        let amount = read_amount(&mut bytes)?;
        // scriptPubKey: CompactSize length + script bytes → hex string
        let script_pubkey = read_script(&mut bytes)?;

        outputs.push(Output { amount, script_pubkey });
    }

    // ── 6. Skip witness data (if SegWit) ─────────────────────
    // Witness is not included in the decoded output here (TXID excludes it)
    if is_segwit {
        for _ in 0..input_count {
            let item_count = read_compact_size(&mut bytes)?;
            for _ in 0..item_count {
                let item_len = read_compact_size(&mut bytes)? as usize;
                read_bytes_n(&mut bytes, item_len)?;
            }
        }
    }

    // ── 7. Locktime (4 bytes LE) ──────────────────────────────
    let lock_time = read_u32(&mut bytes)?;

    // ── 8. Assemble and serialize to JSON ─────────────────────
    let transaction = Transaction {
        transaction_id,
        version,
        inputs,
        outputs,
        lock_time,
    };

    // serde_json::to_string_pretty produces indented JSON
    let json = serde_json::to_string_pretty(&transaction)?;
    Ok(json)
}
