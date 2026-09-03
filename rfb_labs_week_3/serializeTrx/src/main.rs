use clap::Parser;
use std::error::Error;

#[derive(Debug, Clone)]
struct TxInput {
    prev_txid: Vec<u8>,
    vout: u32,
    script_sig: Vec<u8>,
    sequence: u32,
    witness: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
struct TxOutput {
    value: u64,
    script_pubkey: Vec<u8>,
}

#[derive(Debug)]
struct Transaction {
    version: i32,
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
    locktime: u32,
    segwit: bool,
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    if !hex.len().is_multiple_of(2) {
        return Err(format!(
            "hex string '{hex}' must have even length, got {} characters",
            hex.len()
        ));
    }

    // create vector with enough bytes capacity
    let mut bytes = Vec::with_capacity(hex.len() / 2);

    for i in (0..hex.len()).step_by(2) {
        // Give me the next two hexadecimal characters.
        // Convert the two hex characters into a byte
        let byte = u8::from_str_radix(&hex[i..i + 2], 16)
            .map_err(|e| format!("invalid hex string '{hex}': {e}"))?;
        // from_str_radix - Parse a string as a number using a particular base i.e 16
        bytes.push(byte);
    }

    Ok(bytes)
}

/// Parses one `--input` argument.
///
/// Format: `prev_txid_hex:vout:script_sig_hex:sequence[:witness_item_hex,witness_item_hex,...]`
///
/// The witness segment is optional (a legacy input has none) and, when present,
/// is a comma-separated list of hex-encoded witness items. `script_sig` may be
/// an empty string for native SegWit inputs, which carry no scriptSig.
fn parse_input(s: &str) -> Result<TxInput, String> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 4 && parts.len() != 5 {
        return Err(format!(
            "expected 'prev_txid:vout:script_sig:sequence[:witness_items]', got {} field(s) in '{s}'",
            parts.len()
        ));
    }

    let prev_txid = hex_to_bytes(parts[0])?;
    if prev_txid.len() != 32 {
        return Err(format!(
            "prev_txid must be 32 bytes (64 hex characters), got {} bytes",
            prev_txid.len()
        ));
    }

    let vout: u32 = parts[1]
        .parse()
        .map_err(|e| format!("invalid vout '{}': {e}", parts[1]))?;

    let script_sig = hex_to_bytes(parts[2])?;

    let sequence: u32 = parts[3]
        .parse()
        .map_err(|e| format!("invalid sequence '{}': {e}", parts[3]))?;

    let witness = match parts.get(4) {
        None | Some(&"") => vec![],
        Some(items) => items
            .split(',')
            .map(hex_to_bytes)
            .collect::<Result<Vec<Vec<u8>>, String>>()?,
    };

    Ok(TxInput {
        prev_txid,
        vout,
        script_sig,
        sequence,
        witness,
    })
}

/// Parses one `--output` argument.
///
/// Format: `value_sats:script_pubkey_hex`
fn parse_output(s: &str) -> Result<TxOutput, String> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err(format!(
            "expected 'value_sats:script_pubkey', got {} field(s) in '{s}'",
            parts.len()
        ));
    }

    let value: u64 = parts[0]
        .parse()
        .map_err(|e| format!("invalid output value '{}': {e}", parts[0]))?;

    let script_pubkey = hex_to_bytes(parts[1])?;

    Ok(TxOutput {
        value,
        script_pubkey,
    })
}

#[derive(Parser)]
#[command(
    name = "serializetrx",
    disable_version_flag = true,
    about = "Construct and serialize a Bitcoin transaction from command-line arguments",
    long_about = "Construct and serialize a Bitcoin transaction from command-line arguments.\n\n\
--input format:  prev_txid_hex:vout:script_sig_hex:sequence[:witness_item_hex,witness_item_hex,...]\n\
--output format: value_sats:script_pubkey_hex\n\n\
prev_txid must be 32 bytes of hex (64 characters), in internal (non-reversed) byte order.\n\
script_sig may be an empty string for native SegWit inputs.\n\
The witness segment is optional and only used when --segwit is passed."
)]
struct Cli {
    /// Transaction version
    #[arg(long, default_value_t = 2)]
    version: i32,

    /// Mark this as a SegWit transaction (adds marker/flag and serializes witness data)
    #[arg(long)]
    segwit: bool,

    /// One transaction input: prev_txid:vout:script_sig:sequence[:witness_items]
    /// Repeat --input for each input.
    #[arg(long = "input", value_parser = parse_input, required = true)]
    inputs: Vec<TxInput>,

    /// One transaction output: value_sats:script_pubkey
    /// Repeat --output for each output.
    #[arg(long = "output", value_parser = parse_output, required = true)]
    outputs: Vec<TxOutput>,

    /// Transaction locktime
    #[arg(long, default_value_t = 0)]
    locktime: u32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    if !cli.segwit
        && let Some(bad_input) = cli.inputs.iter().find(|i| !i.witness.is_empty())
    {
        return Err(format!(
            "input {} carries witness data but --segwit was not passed",
            bytes_to_hex(&bad_input.prev_txid)
        )
        .into());
    }

    let trx = Transaction {
        version: cli.version,
        inputs: cli.inputs,
        outputs: cli.outputs,
        locktime: cli.locktime,
        segwit: cli.segwit,
    };

    // Serialize
    let serialized = serialize_transaction(&trx);

    println!("Serialized transaction:");
    println!("{:?}", &serialized);
    println!("Serialized Hex transaction:");
    println!("{}", bytes_to_hex(&serialized));

    println!("\nTransaction size: {} bytes", serialized.len());

    Ok(())
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

// ┌──────────────────────────────┐
// │ Version          4 bytes     │
// ├──────────────────────────────┤
// │ Marker           1 byte      │
// │ Flag             1 byte      │
// ├──────────────────────────────┤
// │ Input count      VarInt      │
// │ Inputs           Variable    │
// ├──────────────────────────────┤
// │ Output count     VarInt      │
// │ Outputs          Variable    │
// ├──────────────────────────────┤
// │ Witness          Variable    │
// ├──────────────────────────────┤
// │ Locktime         4 bytes  ←  │
// └──────────────────────────────┘

fn serialize_transaction(trx: &Transaction) -> Vec<u8> {
    let mut result = Vec::new();

    // add version number
    // to_le_bytes: converts the integer into its little-endian byte representation.
    //  extend_from_slice: Take these bytes and append them to result.
    result.extend_from_slice(&trx.version.to_le_bytes());

    if trx.segwit {
        result.push(0x00); // marker
        result.push(0x01); // flag
    };

    // INPUTT COUNT
    // script_sig: vec![] is empty because this particular transaction is a SegWit P2WPKH transaction.
    // scriptSig belongs to the traditional input structure.
    // witness contains the signature and public key for a native SegWit input.
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

// Bitcoin uses VarInts (encode_varint) when it needs to store things like:

// number of inputs
// number of outputs
// script length
// number of witness items
// witness item length

fn encode_varint(value: usize) -> Vec<u8> {
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

// Bitcoin CompactSize follows this structure:
// Value range          Encoding

// 0 - 252              1 byte

// 253 - 65,535         FD + 2 bytes

// 65,536 - 4,294,967,295
//                      FE + 4 bytes

// larger values        FF + 8 bytes

// A simpler way to visualize CompactSize
//               ┌── small value?
//               │
//               ↓
//            0 - 252 (0xfc)
//               │
//               └── store directly
//                     ↓
//                    [XX]

//            253 - 65535
//               │
//               └── FD + 2 bytes
//                     ↓
//                  [FD][XX XX]

//            65536 - 4294967295
//               │
//               └── FE + 4 bytes
//                     ↓
//               [FE][XX XX XX XX]

//            larger
//               │
//               └── FF + 8 bytes
//                     ↓
//           [FF][XX XX XX XX XX XX XX XX]
