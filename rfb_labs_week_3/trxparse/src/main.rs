// Bitcoin Transaction Parser — CLI edition
//
// Companion refactor to the Week 4 serializer. The original program had a
// single raw transaction hex string hardcoded in `main()`. This version
// reads the raw hex from a command-line flag (or a file), validates it
// before decoding, and parses it with the same field-by-field logic as the
// original — now with proper errors instead of `.unwrap()` panics, and
// automatic detection of SegWit vs. legacy transactions (the original
// always assumed SegWit was present).

use byteorder::{LittleEndian, ReadBytesExt};
use clap::Parser;
use std::io::{Cursor, Read};
use std::process::ExitCode;
use thiserror::Error;

// ---------------------------------------------------------------------
// CLI definition
// ---------------------------------------------------------------------

/// Parse and decode a raw Bitcoin transaction supplied via the command line.
#[derive(Parser, Debug)]
#[command(name = "trxparse", version, about, long_about = None)]
struct Cli {
    /// Raw transaction as a hex string
    #[arg(long, value_name = "HEX")]
    tx: Option<String>,

    /// Path to a file containing the raw transaction hex (leading/trailing
    /// whitespace and newlines are trimmed)
    #[arg(long, value_name = "PATH")]
    file: Option<String>,
}

// ---------------------------------------------------------------------
// Error handling
// ---------------------------------------------------------------------

#[derive(Debug, Error)]
enum TxParseError {
    #[error("no transaction hex provided (use --tx <HEX> or --file <PATH>)")]
    NoInput,

    #[error("provide either --tx or --file, not both")]
    ConflictingInput,

    #[error("could not read file '{0}': {1}")]
    FileRead(String, String),

    #[error("invalid hex string: length {0} is odd (hex must have an even number of characters)")]
    OddLengthHex(usize),

    #[error("invalid hex string: contains non-hexadecimal character '{0}'")]
    NonHexCharacter(char),

    #[error("unexpected end of data while reading {0} (transaction hex is truncated or malformed)")]
    UnexpectedEnd(&'static str),

    #[error("invalid SegWit flag byte: expected 0x01, got 0x{0:02x}")]
    InvalidSegwitFlag(u8),

    #[error("{0} unexpected trailing byte(s) after locktime; transaction hex is longer than expected")]
    TrailingBytes(usize),
}

// ---------------------------------------------------------------------
// Hex helpers (validated)
// ---------------------------------------------------------------------

/// Convert a hex string into bytes, validating length parity and that every
/// character is a valid hex digit before converting.
fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, TxParseError> {
    if hex.len() % 2 != 0 {
        return Err(TxParseError::OddLengthHex(hex.len()));
    }

    if let Some(bad) = hex.chars().find(|c| !c.is_ascii_hexdigit()) {
        return Err(TxParseError::NonHexCharacter(bad));
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        // Already validated as hex above, so this cannot fail.
        let byte = u8::from_str_radix(&hex[i..i + 2], 16).unwrap();
        bytes.push(byte);
    }

    Ok(bytes)
}

fn bytes_to_hex(v: &[u8]) -> String {
    v.iter().map(|b| format!("{:02x}", b)).collect()
}

// ---------------------------------------------------------------------
// Low-level cursor readers (validated — no panics on malformed input)
// ---------------------------------------------------------------------

fn read_u8(r: &mut Cursor<Vec<u8>>, field: &'static str) -> Result<u8, TxParseError> {
    r.read_u8().map_err(|_| TxParseError::UnexpectedEnd(field))
}

fn read_u32(r: &mut Cursor<Vec<u8>>, field: &'static str) -> Result<u32, TxParseError> {
    r.read_u32::<LittleEndian>()
        .map_err(|_| TxParseError::UnexpectedEnd(field))
}

fn read_u64(r: &mut Cursor<Vec<u8>>, field: &'static str) -> Result<u64, TxParseError> {
    r.read_u64::<LittleEndian>()
        .map_err(|_| TxParseError::UnexpectedEnd(field))
}

fn read_bytes(r: &mut Cursor<Vec<u8>>, n: usize, field: &'static str) -> Result<Vec<u8>, TxParseError> {
    let mut b = vec![0u8; n];
    r.read_exact(&mut b)
        .map_err(|_| TxParseError::UnexpectedEnd(field))?;
    Ok(b)
}

/// Peek at the next byte without consuming it (used for SegWit marker
/// detection).
fn peek_u8(r: &mut Cursor<Vec<u8>>, field: &'static str) -> Result<u8, TxParseError> {
    let pos = r.position();
    let b = read_u8(r, field)?;
    r.set_position(pos);
    Ok(b)
}

// CompactSize / VarInt reader. Used throughout the Bitcoin transaction
// format to encode input/output counts, script lengths, and witness item
// counts and lengths.
fn read_varint(r: &mut Cursor<Vec<u8>>, field: &'static str) -> Result<u64, TxParseError> {
    let n = read_u8(r, field)?;
    match n {
        0x00..=0xfc => Ok(n as u64),
        0xfd => r
            .read_u16::<LittleEndian>()
            .map(|v| v as u64)
            .map_err(|_| TxParseError::UnexpectedEnd(field)),
        0xfe => read_u32(r, field).map(|v| v as u64),
        _ => read_u64(r, field),
    }
}

// ---------------------------------------------------------------------
// Transaction parsing
// ---------------------------------------------------------------------

// ┌──────────────────────────────┐
// │ Version          4 bytes     │
// ├──────────────────────────────┤
// │ Marker           1 byte      │  (SegWit only)
// │ Flag             1 byte      │  (SegWit only)
// ├──────────────────────────────┤
// │ Input count      VarInt      │
// │ Inputs           Variable    │
// ├──────────────────────────────┤
// │ Output count     VarInt      │
// │ Outputs          Variable    │
// ├──────────────────────────────┤
// │ Witness          Variable    │  (SegWit only)
// ├──────────────────────────────┤
// │ Locktime         4 bytes  ←  │
// └──────────────────────────────┘

fn parse_transaction(raw_hex: &str) -> Result<(), TxParseError> {
    let bytes = hex_to_bytes(raw_hex)?;
    let total_len = bytes.len();
    let mut r = Cursor::new(bytes);

    // Version: 4-byte little-endian integer.
    let version = read_u32(&mut r, "version")?;
    println!("Version: {}", version);

    // SegWit detection: a legacy transaction's next byte is the input-count
    // VarInt (never 0x00 for a valid transaction, since the input count is
    // always at least 1). A SegWit transaction inserts marker=0x00 and a
    // flag byte before the input count. Peek to decide which we have,
    // instead of assuming SegWit unconditionally.
    let maybe_marker = peek_u8(&mut r, "segwit marker")?;
    let is_segwit = maybe_marker == 0x00;

    if is_segwit {
        let marker = read_u8(&mut r, "segwit marker")?;
        let flag = read_u8(&mut r, "segwit flag")?;
        if flag != 0x01 {
            return Err(TxParseError::InvalidSegwitFlag(flag));
        }
        println!("SegWit marker={} flag={}", marker, flag);
    } else {
        println!("SegWit: no (legacy transaction)");
    }

    let in_count = read_varint(&mut r, "input count")?;
    println!("Inputs: {}", in_count);

    for i in 0..in_count {
        println!("Input {}", i);

        // 32-byte previous transaction ID.
        let prev = read_bytes(&mut r, 32, "input previous txid")?;
        println!("  Prev TXID (LE): {}", bytes_to_hex(&prev));

        let vout = read_u32(&mut r, "input vout")?;
        println!("  Vout: {}", vout);

        let slen = read_varint(&mut r, "input scriptSig length")? as usize;
        println!("  script length: {}", slen);

        let script = read_bytes(&mut r, slen, "input scriptSig")?;
        println!("  ScriptSig: {}", bytes_to_hex(&script));

        let seq = read_u32(&mut r, "input sequence")?;
        println!("  Sequence: {:08x}", seq);
    }

    let out_count = read_varint(&mut r, "output count")?;
    println!("Outputs: {}", out_count);

    for i in 0..out_count {
        println!("Output {}", i);

        let value = read_u64(&mut r, "output value")?;
        println!("  Value: {} sats", value);

        let slen = read_varint(&mut r, "output scriptPubKey length")? as usize;
        println!("  script length: {}", slen);

        let script = read_bytes(&mut r, slen, "output scriptPubKey")?;
        println!("  ScriptPubKey: {}", bytes_to_hex(&script));
    }

    // Each input has its own witness field, present only for SegWit
    // transactions.
    if is_segwit {
        for i in 0..in_count {
            let items = read_varint(&mut r, "witness item count")?;
            println!("Witness for input {} ({} item(s))", i, items);
            for j in 0..items {
                let len = read_varint(&mut r, "witness item length")? as usize;
                let item = read_bytes(&mut r, len, "witness item")?;
                println!("  Item {}: {}", j, bytes_to_hex(&item));
            }
        }
    }

    let locktime = read_u32(&mut r, "locktime")?;
    println!("Locktime: {}", locktime);

    let remaining = total_len as u64 - r.position();
    if remaining > 0 {
        return Err(TxParseError::TrailingBytes(remaining as usize));
    }

    Ok(())
}

// ---------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------

fn load_raw_hex(cli: &Cli) -> Result<String, TxParseError> {
    match (&cli.tx, &cli.file) {
        (Some(_), Some(_)) => Err(TxParseError::ConflictingInput),
        (None, None) => Err(TxParseError::NoInput),
        (Some(tx), None) => Ok(tx.trim().to_string()),
        (None, Some(path)) => {
            let content = std::fs::read_to_string(path)
                .map_err(|e| TxParseError::FileRead(path.clone(), e.to_string()))?;
            Ok(content.trim().to_string())
        }
    }
}

fn run() -> Result<(), TxParseError> {
    let cli = Cli::parse();
    let raw_hex = load_raw_hex(&cli)?;
    parse_transaction(&raw_hex)
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {}", e);
            ExitCode::FAILURE
        }
    }
}