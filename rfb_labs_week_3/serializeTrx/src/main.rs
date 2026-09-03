use clap::{Arg, Command};
use serde_json::Value;
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

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if hex.is_empty() {
        return Ok(Vec::new());
    }

    if hex.len() % 2 != 0 {
        return Err("Hex string must have even length".into());
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);

    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16)?;
        bytes.push(byte);
    }

    Ok(bytes)
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn parse_input_json(json_str: &str) -> Result<TxInput, Box<dyn Error>> {
    let obj: Value = serde_json::from_str(json_str)?;

    let prev_txid = hex_to_bytes(
        obj.get("prev_txid")
            .and_then(|v| v.as_str())
            .ok_or("Missing or invalid prev_txid")?,
    )?;

    if prev_txid.len() != 32 {
        return Err("prev_txid must be exactly 32 bytes (64 hex characters)".into());
    }

    let vout = obj
        .get("vout")
        .and_then(|v| v.as_u64())
        .ok_or("Missing or invalid vout")?;

    if vout > u32::MAX as u64 {
        return Err("vout must fit in u32".into());
    }

    let script_sig = hex_to_bytes(obj.get("script_sig").and_then(|v| v.as_str()).unwrap_or(""))?;

    let sequence = obj
        .get("sequence")
        .and_then(|v| v.as_u64())
        .unwrap_or(0xffffffff) as u32;

    let witness: Vec<Vec<u8>> =
        if let Some(witness_array) = obj.get("witness").and_then(|v| v.as_array()) {
            witness_array
                .iter()
                .map(|item| hex_to_bytes(item.as_str().ok_or("Witness item must be a hex string")?))
                .collect::<Result<Vec<_>, _>>()?
        } else {
            Vec::new()
        };

    Ok(TxInput {
        prev_txid,
        vout: vout as u32,
        script_sig,
        sequence,
        witness,
    })
}

fn parse_output_json(json_str: &str) -> Result<TxOutput, Box<dyn Error>> {
    let obj: Value = serde_json::from_str(json_str)?;

    let value = obj
        .get("value")
        .and_then(|v| v.as_u64())
        .ok_or("Missing or invalid value")?;

    let script_pubkey = hex_to_bytes(
        obj.get("script_pubkey")
            .and_then(|v| v.as_str())
            .ok_or("Missing or invalid script_pubkey")?,
    )?;

    Ok(TxOutput {
        value,
        script_pubkey,
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("Bitcoin Transaction Serializer")
        .version("0.1.0")
        .author("Bitcoin Labs")
        .about("Serializes Bitcoin transactions from command-line arguments")
        .disable_version_flag(true)
        .arg(
            Arg::new("version")
                .long("version")
                .value_name("VERSION")
                .help("Transaction version (default: 2)")
                .default_value("2"),
        )
        .arg(
            Arg::new("segwit")
                .long("segwit")
                .help("Enable SegWit serialization")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("locktime")
                .long("locktime")
                .value_name("LOCKTIME")
                .help("Transaction locktime (default: 0)")
                .default_value("0"),
        )
        .arg(
            Arg::new("input")
                .long("input")
                .value_name("JSON")
                .help("Transaction input as JSON. Can be used multiple times. JSON format: {\"prev_txid\":\"...\",\"vout\":0,\"script_sig\":\"\",\"sequence\":4294967295,\"witness\":[]}")
                .action(clap::ArgAction::Append),
        )
        .arg(
            Arg::new("output")
                .long("output")
                .value_name("JSON")
                .help("Transaction output as JSON. Can be used multiple times. JSON format: {\"value\":0,\"script_pubkey\":\"...\"}")
                .action(clap::ArgAction::Append),
        )
        .get_matches();

    // Parse version
    let version: i32 = matches
        .get_one::<String>("version")
        .and_then(|v| v.parse().ok())
        .ok_or("Invalid version number")?;

    // Parse SegWit flag
    let segwit = matches.get_flag("segwit");

    // Parse locktime
    let locktime: u32 = matches
        .get_one::<String>("locktime")
        .and_then(|v| v.parse().ok())
        .ok_or("Invalid locktime")?;

    // Parse inputs
    let mut inputs = Vec::new();
    if let Some(input_strs) = matches.get_many::<String>("input") {
        for input_str in input_strs {
            inputs.push(parse_input_json(input_str)?);
        }
    }

    // Parse outputs
    let mut outputs = Vec::new();
    if let Some(output_strs) = matches.get_many::<String>("output") {
        for output_str in output_strs {
            outputs.push(parse_output_json(output_str)?);
        }
    }

    // Validate transaction has at least one input and one output
    if inputs.is_empty() {
        return Err("Transaction must have at least one input".into());
    }
    if outputs.is_empty() {
        return Err("Transaction must have at least one output".into());
    }

    // Create transaction
    let trx = Transaction {
        version,
        inputs,
        outputs,
        locktime,
        segwit,
    };

    // Serialize
    let serialized = serialize_transaction(&trx);

    // Display results
    println!("\n=== Bitcoin Transaction Serialization ===\n");
    println!("Transaction Details:");
    println!("  Version: {}", trx.version);
    println!("  SegWit: {}", trx.segwit);
    println!("  Input Count: {}", trx.inputs.len());
    println!("  Output Count: {}", trx.outputs.len());
    println!("  Locktime: {}", trx.locktime);

    println!("\nSerialized Transaction (Hex):");
    println!("{}", bytes_to_hex(&serialized));

    println!("\nTransaction Size: {} bytes", serialized.len());

    Ok(())
}

fn serialize_transaction(trx: &Transaction) -> Vec<u8> {
    let mut result = Vec::new();

    // Add version number
    result.extend_from_slice(&trx.version.to_le_bytes());

    // Add SegWit marker and flag if applicable
    if trx.segwit {
        result.push(0x00); // marker
        result.push(0x01); // flag
    }

    // Input count
    result.extend_from_slice(&encode_varint(trx.inputs.len()));

    // Input data
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

    // Output count
    result.extend_from_slice(&encode_varint(trx.outputs.len()));

    // Output data
    for output in &trx.outputs {
        // Value in satoshis
        result.extend_from_slice(&output.value.to_le_bytes());

        // ScriptPubKey length
        result.extend_from_slice(&encode_varint(output.script_pubkey.len()));

        // ScriptPubKey
        result.extend_from_slice(&output.script_pubkey);
    }

    // Witness data (if SegWit)
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

    // Add locktime
    result.extend_from_slice(&trx.locktime.to_le_bytes());

    result
}

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
