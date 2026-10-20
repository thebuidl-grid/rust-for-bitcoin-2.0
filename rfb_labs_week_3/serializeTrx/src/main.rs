use clap::Parser;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TxInput {
    prev_txid: Vec<u8>,
    vout: u32,
    script_sig: Vec<u8>,
    sequence: u32,
    witness: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TxOutput {
    value: u64,
    script_pubkey: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    version: i32,
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
    locktime: u32,
    segwit: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct JsonInput {
    prev_txid: String,
    vout: u32,
    #[serde(default)]
    script_sig: String,
    #[serde(default = "default_sequence")]
    sequence: u32,
    #[serde(default)]
    witness: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct JsonOutput {
    value: u64,
    script_pubkey: String,
}

#[derive(Debug, Clone, Deserialize)]
struct JsonTransaction {
    #[serde(default = "default_version")]
    version: i32,
    inputs: Vec<JsonInput>,
    outputs: Vec<JsonOutput>,
    #[serde(default)]
    locktime: u32,
    #[serde(default)]
    segwit: Option<bool>,
}

fn default_version() -> i32 {
    2
}

fn default_sequence() -> u32 {
    0xffffffff
}

#[derive(Parser, Debug)]
#[command(
    name = "serializeTrx",
    version = "1.0",
    about = "Construct and serialize Bitcoin transactions via CLI arguments or JSON configuration",
    disable_version_flag = true
)]
struct Cli {
    /// Read transaction definition from a JSON file
    #[arg(short, long, value_name = "FILE")]
    file: Option<PathBuf>,

    /// Read transaction definition from a JSON string
    #[arg(short, long, value_name = "JSON")]
    json: Option<String>,

    /// Transaction version (e.g., 1 or 2)
    #[arg(short = 'V', long = "version", default_value_t = 2)]
    version: i32,

    /// Locktime (e.g., 0)
    #[arg(short, long, default_value_t = 0)]
    locktime: u32,

    /// Explicitly enable SegWit formatting
    #[arg(long)]
    segwit: bool,

    /// Add an input formatted as: PREV_TXID:VOUT[:SEQUENCE][:SCRIPT_SIG_HEX]
    /// Example: 8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821:1:4294967295
    #[arg(short, long = "input", value_name = "INPUT_SPEC")]
    inputs: Vec<String>,

    /// Add witness item(s) for an input formatted as: INPUT_INDEX:ITEM1_HEX[,ITEM2_HEX...]
    /// Example: 0:3045022100...,029cbb...
    #[arg(short, long = "witness", value_name = "WITNESS_SPEC")]
    witnesses: Vec<String>,

    /// Add an output formatted as: VALUE_SATS:SCRIPT_PUBKEY_HEX
    /// Example: 69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b
    #[arg(short, long = "output", value_name = "OUTPUT_SPEC")]
    outputs: Vec<String>,
}

#[allow(clippy::manual_is_multiple_of)]
fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if hex.len() % 2 != 0 {
        return Err("Hex string must have even length".into());
    }

    // create vector with enough bytes capacity
    let mut bytes = Vec::with_capacity(hex.len() / 2);

    for i in (0..hex.len()).step_by(2) {
        // Give me the next two hexadecimal characters.
        // Convert the two hex characters into a byte
        let byte = u8::from_str_radix(&hex[i..i + 2], 16)?;
        // from_str_radix - Parse a string as a number using a particular base i.e 16
        bytes.push(byte);
    }

    Ok(bytes)
}

fn validate_txid(txid_hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let clean = txid_hex.trim();
    if clean.len() != 64 {
        return Err(format!(
            "Invalid TXID '{clean}': expected exactly 64 hexadecimal characters (32 bytes), got {}",
            clean.len()
        )
        .into());
    }
    hex_to_bytes(clean)
}

fn parse_cli_inputs(
    raw_inputs: &[String],
    raw_witnesses: &[String],
) -> Result<Vec<TxInput>, Box<dyn Error>> {
    if raw_inputs.is_empty() {
        return Err("Transaction must contain at least one input (--input)".into());
    }

    let mut inputs = Vec::new();
    for (idx, spec) in raw_inputs.iter().enumerate() {
        let parts: Vec<&str> = spec.split(':').collect();
        if parts.len() < 2 {
            return Err(format!(
                "Invalid input spec '{spec}' at index {idx}. Expected format: PREV_TXID:VOUT[:SEQUENCE][:SCRIPT_SIG_HEX]"
            )
            .into());
        }

        let prev_txid = validate_txid(parts[0])?;
        let vout: u32 = parts[1]
            .parse()
            .map_err(|_| format!("Invalid vout index '{}' in input spec '{spec}'", parts[1]))?;

        let sequence: u32 = if parts.len() >= 3 && !parts[2].is_empty() {
            parts[2].parse().map_err(|_| {
                format!(
                    "Invalid sequence number '{}' in input spec '{spec}'",
                    parts[2]
                )
            })?
        } else {
            0xffffffff
        };

        let script_sig = if parts.len() >= 4 && !parts[3].is_empty() {
            hex_to_bytes(parts[3])?
        } else {
            Vec::new()
        };

        inputs.push(TxInput {
            prev_txid,
            vout,
            script_sig,
            sequence,
            witness: Vec::new(),
        });
    }

    for wit_spec in raw_witnesses {
        let parts: Vec<&str> = wit_spec.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(format!(
                "Invalid witness spec '{wit_spec}'. Expected format: INPUT_INDEX:ITEM1_HEX[,ITEM2_HEX...]"
            )
            .into());
        }

        let input_idx: usize = parts[0]
            .parse()
            .map_err(|_| format!("Invalid input index '{}' in witness spec", parts[0]))?;

        if input_idx >= inputs.len() {
            return Err(format!(
                "Witness specifies input index {input_idx}, but only {} input(s) were defined",
                inputs.len()
            )
            .into());
        }

        let items_hex: Vec<&str> = parts[1].split(',').collect();
        let mut witness_items = Vec::new();
        for item_hex in items_hex {
            if !item_hex.trim().is_empty() {
                witness_items.push(hex_to_bytes(item_hex.trim())?);
            }
        }
        inputs[input_idx].witness = witness_items;
    }

    Ok(inputs)
}

fn parse_cli_outputs(raw_outputs: &[String]) -> Result<Vec<TxOutput>, Box<dyn Error>> {
    if raw_outputs.is_empty() {
        return Err("Transaction must contain at least one output (--output)".into());
    }

    let mut outputs = Vec::new();
    for (idx, spec) in raw_outputs.iter().enumerate() {
        let parts: Vec<&str> = spec.split(':').collect();
        if parts.len() != 2 {
            return Err(format!(
                "Invalid output spec '{spec}' at index {idx}. Expected format: VALUE_SATS:SCRIPT_PUBKEY_HEX"
            )
            .into());
        }

        let value: u64 = parts[0].parse().map_err(|_| {
            format!(
                "Invalid satoshi amount '{}' in output spec '{spec}'",
                parts[0]
            )
        })?;

        let script_pubkey = hex_to_bytes(parts[1])?;

        outputs.push(TxOutput {
            value,
            script_pubkey,
        });
    }

    Ok(outputs)
}

fn parse_json_transaction(json_str: &str) -> Result<Transaction, Box<dyn Error>> {
    let parsed: JsonTransaction = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse JSON transaction: {e}"))?;

    if parsed.inputs.is_empty() {
        return Err("Transaction JSON must contain at least one input".into());
    }
    if parsed.outputs.is_empty() {
        return Err("Transaction JSON must contain at least one output".into());
    }

    let mut inputs = Vec::new();
    let mut has_witness_data = false;

    for inp in parsed.inputs {
        let prev_txid = validate_txid(&inp.prev_txid)?;
        let script_sig = if inp.script_sig.trim().is_empty() {
            Vec::new()
        } else {
            hex_to_bytes(&inp.script_sig)?
        };

        let mut witness = Vec::new();
        for wit_item in inp.witness {
            if !wit_item.trim().is_empty() {
                has_witness_data = true;
                witness.push(hex_to_bytes(&wit_item)?);
            }
        }

        inputs.push(TxInput {
            prev_txid,
            vout: inp.vout,
            script_sig,
            sequence: inp.sequence,
            witness,
        });
    }

    let mut outputs = Vec::new();
    for out in parsed.outputs {
        let script_pubkey = hex_to_bytes(&out.script_pubkey)?;
        outputs.push(TxOutput {
            value: out.value,
            script_pubkey,
        });
    }

    let segwit = parsed.segwit.unwrap_or(has_witness_data);

    Ok(Transaction {
        version: parsed.version,
        inputs,
        outputs,
        locktime: parsed.locktime,
        segwit,
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let trx = if let Some(path) = cli.file {
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read JSON file '{path:?}': {e}"))?;
        parse_json_transaction(&content)?
    } else if let Some(json_str) = cli.json {
        parse_json_transaction(&json_str)?
    } else {
        let inputs = parse_cli_inputs(&cli.inputs, &cli.witnesses)?;
        let outputs = parse_cli_outputs(&cli.outputs)?;
        let has_witness = inputs.iter().any(|i| !i.witness.is_empty());
        let segwit = cli.segwit || has_witness;

        Transaction {
            version: cli.version,
            inputs,
            outputs,
            locktime: cli.locktime,
            segwit,
        }
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
    bytes.iter().map(|b| format!("{b:02x}")).collect()
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
    // extend_from_slice: Take these bytes and append them to result.
    result.extend_from_slice(&trx.version.to_le_bytes());

    if trx.segwit {
        result.push(0x00); // marker
        result.push(0x01); // flag
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_bytes_valid() {
        assert_eq!(hex_to_bytes("0014a6").unwrap(), vec![0x00, 0x14, 0xa6]);
    }

    #[test]
    fn test_hex_to_bytes_invalid_length() {
        assert!(hex_to_bytes("0014a").is_err());
    }

    #[test]
    fn test_hex_to_bytes_invalid_character() {
        assert!(hex_to_bytes("0014z6").is_err());
    }

    #[test]
    fn test_txid_validation() {
        assert!(
            validate_txid("8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821")
                .is_ok()
        );
        assert!(validate_txid("short_txid").is_err());
    }

    #[test]
    fn test_serialization_matches_expected() {
        let input = TxInput {
            prev_txid: hex_to_bytes(
                "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821",
            )
            .unwrap(),
            vout: 1,
            script_sig: vec![],
            sequence: 0xffffffff,
            witness: vec![
                hex_to_bytes("3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301").unwrap(),
                hex_to_bytes("029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358").unwrap(),
            ],
        };

        let output_0 = TxOutput {
            value: 69886,
            script_pubkey: hex_to_bytes("0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b").unwrap(),
        };

        let output_1 = TxOutput {
            value: 29442,
            script_pubkey: hex_to_bytes("00149831122b93d21715c70db626ccc844d3c21f9687").unwrap(),
        };

        let trx = Transaction {
            version: 2,
            inputs: vec![input],
            outputs: vec![output_0, output_1],
            locktime: 0,
            segwit: true,
        };

        let serialized = serialize_transaction(&trx);
        let hex_str = bytes_to_hex(&serialized);
        assert_eq!(serialized.len(), 223);
        assert!(hex_str.starts_with("020000000001018fb0d07b"));
    }
}
