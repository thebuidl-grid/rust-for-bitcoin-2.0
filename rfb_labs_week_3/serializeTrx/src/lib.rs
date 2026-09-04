//! Bitcoin transaction serializer.
//!
//! Builds a raw Bitcoin transaction from values supplied on the command line
//! and serializes it to bytes.
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

use std::fmt;

/// 21 million BTC, in satoshis. No output may exceed this.
pub const MAX_MONEY: u64 = 21_000_000 * 100_000_000;

/// Default sequence — signals "not using relative locktime".
pub const SEQUENCE_FINAL: u32 = 0xffff_ffff;

#[derive(Debug)]
pub struct TxInput {
    /// Previous txid in *internal* byte order, ready to write to the wire.
    pub prev_txid: Vec<u8>,
    pub vout: u32,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
    pub witness: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct TxOutput {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
}

#[derive(Debug)]
pub struct Transaction {
    pub version: i32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub locktime: u32,
    pub segwit: bool,
}

/// Which byte order a txid was supplied in.
///
/// Explorers and RPC display txids reversed from how they sit on the wire, so
/// a value copied from mempool.space is in [`TxidOrder::Display`] order and has
/// to be reversed before serializing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxidOrder {
    /// As shown by block explorers. Reversed before writing. This is the default.
    Display,
    /// Already in wire order. Written as-is.
    Internal,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SerializeError {
    OddLengthHex { field: String, length: usize },
    InvalidHexDigit { field: String, value: String },
    BadTxidLength { length: usize },
    MalformedSpec { flag: String, spec: String, expected: String },
    NotANumber { field: String, value: String },
    AmountTooLarge { value: u64 },
    WitnessIndexOutOfRange { index: usize, input_count: usize },
    WitnessWithoutSegwit,
    SegwitWithoutWitness,
    DuplicateWitness { index: usize },
}

impl fmt::Display for SerializeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OddLengthHex { field, length } => write!(
                f,
                "{}: hex must have an even number of characters, got {}",
                field, length
            ),
            Self::InvalidHexDigit { field, value } => {
                write!(f, "{}: '{}' is not valid hexadecimal", field, value)
            }
            Self::BadTxidLength { length } => write!(
                f,
                "txid must be exactly 32 bytes (64 hex characters), got {} bytes",
                length
            ),
            Self::MalformedSpec { flag, spec, expected } => {
                write!(f, "{} '{}': expected {}", flag, spec, expected)
            }
            Self::NotANumber { field, value } => {
                write!(f, "{}: '{}' is not a valid number", field, value)
            }
            Self::AmountTooLarge { value } => write!(
                f,
                "output amount {} sats exceeds the 21 million BTC supply cap ({} sats)",
                value, MAX_MONEY
            ),
            Self::WitnessIndexOutOfRange { index, input_count } => write!(
                f,
                "--witness refers to input {} but only {} input(s) were supplied",
                index, input_count
            ),
            Self::WitnessWithoutSegwit => write!(
                f,
                "--witness was supplied without --segwit; witness data only exists in SegWit transactions"
            ),
            Self::SegwitWithoutWitness => write!(
                f,
                "--segwit was supplied but no --witness data; BIP144 forbids the marker and flag when there is no witness"
            ),
            Self::DuplicateWitness { index } => {
                write!(f, "--witness given more than once for input {}", index)
            }
        }
    }
}

impl std::error::Error for SerializeError {}

/// Converts a hex string into bytes, rejecting odd lengths and non-hex digits.
pub fn hex_to_bytes(hex: &str, field: &str) -> Result<Vec<u8>, SerializeError> {
    let hex = hex.trim();

    if hex.len() % 2 != 0 {
        return Err(SerializeError::OddLengthHex {
            field: field.to_string(),
            length: hex.len(),
        });
    }

    // create vector with enough bytes capacity
    let mut bytes = Vec::with_capacity(hex.len() / 2);

    for i in (0..hex.len()).step_by(2) {
        // Give me the next two hexadecimal characters.
        // Convert the two hex characters into a byte.
        // from_str_radix - Parse a string as a number using a particular base i.e 16
        let pair = &hex[i..i + 2];
        let byte = u8::from_str_radix(pair, 16).map_err(|_| SerializeError::InvalidHexDigit {
            field: field.to_string(),
            value: pair.to_string(),
        })?;
        bytes.push(byte);
    }

    Ok(bytes)
}

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Parses an unsigned integer, accepting either decimal or a `0x` hex prefix.
fn parse_number(value: &str, field: &str) -> Result<u64, SerializeError> {
    let value = value.trim();
    let parsed = if let Some(hex) = value.strip_prefix("0x").or_else(|| value.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16)
    } else {
        value.parse::<u64>()
    };

    parsed.map_err(|_| SerializeError::NotANumber {
        field: field.to_string(),
        value: value.to_string(),
    })
}

/// Parses `txid:vout[:script_sig_hex[:sequence]]`.
///
/// `script_sig` defaults to empty and `sequence` to `0xffffffff`.
pub fn parse_input(spec: &str, order: TxidOrder) -> Result<TxInput, SerializeError> {
    let parts: Vec<&str> = spec.split(':').collect();

    if parts.len() < 2 || parts.len() > 4 {
        return Err(SerializeError::MalformedSpec {
            flag: "--input".to_string(),
            spec: spec.to_string(),
            expected: "txid:vout[:script_sig_hex[:sequence]]".to_string(),
        });
    }

    let mut prev_txid = hex_to_bytes(parts[0], "txid")?;
    if prev_txid.len() != 32 {
        return Err(SerializeError::BadTxidLength {
            length: prev_txid.len(),
        });
    }
    // Explorers show txids reversed from wire order, so a pasted value needs
    // flipping before it goes into the transaction.
    if order == TxidOrder::Display {
        prev_txid.reverse();
    }

    let vout = parse_number(parts[1], "vout")?;
    let vout = u32::try_from(vout).map_err(|_| SerializeError::NotANumber {
        field: "vout".to_string(),
        value: parts[1].to_string(),
    })?;

    let script_sig = match parts.get(2) {
        Some(hex) if !hex.is_empty() => hex_to_bytes(hex, "script_sig")?,
        _ => Vec::new(),
    };

    let sequence = match parts.get(3) {
        Some(value) if !value.is_empty() => {
            let sequence = parse_number(value, "sequence")?;
            u32::try_from(sequence).map_err(|_| SerializeError::NotANumber {
                field: "sequence".to_string(),
                value: value.to_string(),
            })?
        }
        _ => SEQUENCE_FINAL,
    };

    Ok(TxInput {
        prev_txid,
        vout,
        script_sig,
        sequence,
        witness: Vec::new(),
    })
}

/// Parses `amount_sats:script_pubkey_hex`.
pub fn parse_output(spec: &str) -> Result<TxOutput, SerializeError> {
    let parts: Vec<&str> = spec.split(':').collect();

    if parts.len() != 2 {
        return Err(SerializeError::MalformedSpec {
            flag: "--output".to_string(),
            spec: spec.to_string(),
            expected: "amount_sats:script_pubkey_hex".to_string(),
        });
    }

    let value = parse_number(parts[0], "amount")?;
    if value > MAX_MONEY {
        return Err(SerializeError::AmountTooLarge { value });
    }

    Ok(TxOutput {
        value,
        script_pubkey: hex_to_bytes(parts[1], "script_pubkey")?,
    })
}

/// Parses `input_index:item_hex[,item_hex...]`.
///
/// An empty item list is allowed, and means an explicitly empty witness stack.
pub fn parse_witness(spec: &str) -> Result<(usize, Vec<Vec<u8>>), SerializeError> {
    let (index, items) = spec.split_once(':').ok_or_else(|| SerializeError::MalformedSpec {
        flag: "--witness".to_string(),
        spec: spec.to_string(),
        expected: "input_index:item_hex[,item_hex...]".to_string(),
    })?;

    let index = parse_number(index, "witness input index")? as usize;

    let items = items
        .split(',')
        .filter(|item| !item.is_empty())
        .map(|item| hex_to_bytes(item, "witness item"))
        .collect::<Result<Vec<_>, _>>()?;

    Ok((index, items))
}

/// Assembles a transaction from parsed specs, checking the pieces agree.
pub fn build_transaction(
    version: i32,
    segwit: bool,
    mut inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
    witnesses: Vec<(usize, Vec<Vec<u8>>)>,
    locktime: u32,
) -> Result<Transaction, SerializeError> {
    if !segwit && !witnesses.is_empty() {
        return Err(SerializeError::WitnessWithoutSegwit);
    }
    if segwit && witnesses.is_empty() {
        return Err(SerializeError::SegwitWithoutWitness);
    }

    let input_count = inputs.len();
    let mut seen = vec![false; input_count];

    for (index, items) in witnesses {
        if index >= input_count {
            return Err(SerializeError::WitnessIndexOutOfRange { index, input_count });
        }
        if seen[index] {
            return Err(SerializeError::DuplicateWitness { index });
        }
        seen[index] = true;
        inputs[index].witness = items;
    }

    Ok(Transaction {
        version,
        inputs,
        outputs,
        locktime,
        segwit,
    })
}

// Bitcoin uses VarInts (encode_varint) when it needs to store things like:
//   number of inputs, number of outputs, script length,
//   number of witness items, witness item length.
//
// Bitcoin CompactSize follows this structure:
//   0 - 252                  1 byte
//   253 - 65,535             FD + 2 bytes
//   65,536 - 4,294,967,295   FE + 4 bytes
//   larger values            FF + 8 bytes
pub fn encode_varint(value: usize) -> Vec<u8> {
    match value {
        0..=0xfc => vec![value as u8],

        0xfd..=0xffff => {
            let mut result = vec![0xfd];
            result.extend_from_slice(&(value as u16).to_le_bytes());
            result
        }

        0x10000..=0xffff_ffff => {
            let mut result = vec![0xfe];
            result.extend_from_slice(&(value as u32).to_le_bytes());
            result
        }

        _ => {
            let mut result = vec![0xff];
            result.extend_from_slice(&(value as u64).to_le_bytes());
            result
        }
    }
}

pub fn serialize_transaction(trx: &Transaction) -> Vec<u8> {
    let mut result = Vec::new();

    // add version number
    // to_le_bytes: converts the integer into its little-endian byte representation.
    // extend_from_slice: Take these bytes and append them to result.
    result.extend_from_slice(&trx.version.to_le_bytes());

    if trx.segwit {
        result.push(0x00); // marker
        result.push(0x01); // flag
    };

    // INPUT COUNT
    result.extend_from_slice(&encode_varint(trx.inputs.len()));

    // input data
    for input in &trx.inputs {
        // Previous transaction ID
        result.extend_from_slice(&input.prev_txid);

        // Previous output index
        result.extend_from_slice(&input.vout.to_le_bytes());

        // ScriptSig length
        result.extend_from_slice(&encode_varint(input.script_sig.len()));

        // ScriptSig
        // For a native SegWit input this is empty: the signature and public key
        // live in the witness instead.
        result.extend_from_slice(&input.script_sig);

        // Sequence
        result.extend_from_slice(&input.sequence.to_le_bytes());
    }

    // OUTPUT COUNT
    result.extend_from_slice(&encode_varint(trx.outputs.len()));

    // OUTPUT DATA
    for output in &trx.outputs {
        // Value in satoshis
        result.extend_from_slice(&output.value.to_le_bytes());

        // ScriptPubKey length
        result.extend_from_slice(&encode_varint(output.script_pubkey.len()));

        // ScriptPubKey
        result.extend_from_slice(&output.script_pubkey);
    }

    // witness data
    if trx.segwit {
        for input in &trx.inputs {
            // Number of witness items
            result.extend_from_slice(&encode_varint(input.witness.len()));

            for item in &input.witness {
                // Witness item length
                result.extend_from_slice(&encode_varint(item.len()));

                // Witness item
                result.extend_from_slice(item);
            }
        }
    }

    // add locktime
    result.extend_from_slice(&trx.locktime.to_le_bytes());

    result
}
