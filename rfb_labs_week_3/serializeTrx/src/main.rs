use std::error::Error;
use clap::Parser;
use serde_json::Value;

#[derive(Debug)]
struct TxInput {
    prev_txid: Vec<u8>,
    vout: u32,
    script_sig: Vec<u8>,
    sequence: u32,
    witness: Vec<Vec<u8>>,
}

#[derive(Debug)]
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

#[derive(Parser, Debug)]
#[command(name = "Bitcoin Transaction Serializer")]
#[command(about = "Serialize Bitcoin transactions from command-line arguments", long_about = None)]
struct Args {
    /// Transaction version (default: 2)
    #[arg(long, default_value = "2")]
    version: i32,

    /// Enable SegWit serialization (use flag without value)
    #[arg(long, action = clap::ArgAction::SetTrue)]
    segwit: bool,

    /// Locktime value (default: 0)
    #[arg(long, default_value = "0")]
    locktime: u32,

    /// JSON array of inputs. Format: [{"prev_txid":"hex","vout":0,"script_sig":"hex","sequence":4294967295,"witness":["hex"]}]
    #[arg(long)]
    inputs: String,

    /// JSON array of outputs. Format: [{"value":69886,"script_pubkey":"hex"}]
    #[arg(long)]
    outputs: String,
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let hex = hex.trim();

    if hex.is_empty() {
        return Ok(Vec::new());
    }

    if hex.len() % 2 != 0 {
        return Err(format!("Hex string '{}' has odd length", hex).into());
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);

    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16)
            .map_err(|e| format!("Invalid hex in '{}': {}", hex, e))?;
        bytes.push(byte);
    }

    Ok(bytes)
}

fn parse_inputs(json_str: &str) -> Result<Vec<TxInput>, Box<dyn Error>> {
    let value: Value = serde_json::from_str(json_str)?;
    let inputs_array = value.as_array()
        .ok_or("Inputs must be a JSON array")?;

    let mut inputs = Vec::new();

    for (idx, input_obj) in inputs_array.iter().enumerate() {
        let obj = input_obj.as_object()
            .ok_or(format!("Input {} must be an object", idx))?;

        let prev_txid_hex = obj.get("prev_txid")
            .and_then(|v| v.as_str())
            .ok_or(format!("Input {} missing 'prev_txid' (string)", idx))?;

        let prev_txid = hex_to_bytes(prev_txid_hex)?;

        let vout = obj.get("vout")
            .and_then(|v| v.as_u64())
            .ok_or(format!("Input {} missing 'vout' (number)", idx))? as u32;

        let script_sig_hex = obj.get("script_sig")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let script_sig = hex_to_bytes(script_sig_hex)?;

        let sequence = obj.get("sequence")
            .and_then(|v| v.as_u64())
            .unwrap_or(0xffffffff) as u32;

        let witness = if let Some(witness_array) = obj.get("witness").and_then(|v| v.as_array()) {
            let mut w = Vec::new();
            for (widx, witness_item) in witness_array.iter().enumerate() {
                let hex_str = witness_item.as_str()
                    .ok_or(format!("Input {} witness item {} must be a hex string", idx, widx))?;
                w.push(hex_to_bytes(hex_str)?);
            }
            w
        } else {
            Vec::new()
        };

        inputs.push(TxInput {
            prev_txid,
            vout,
            script_sig,
            sequence,
            witness,
        });
    }

    Ok(inputs)
}

fn parse_outputs(json_str: &str) -> Result<Vec<TxOutput>, Box<dyn Error>> {
    let value: Value = serde_json::from_str(json_str)?;
    let outputs_array = value.as_array()
        .ok_or("Outputs must be a JSON array")?;

    let mut outputs = Vec::new();

    for (idx, output_obj) in outputs_array.iter().enumerate() {
        let obj = output_obj.as_object()
            .ok_or(format!("Output {} must be an object", idx))?;

        let value = obj.get("value")
            .and_then(|v| v.as_u64())
            .ok_or(format!("Output {} missing 'value' (number)", idx))?;

        let script_pubkey_hex = obj.get("script_pubkey")
            .and_then(|v| v.as_str())
            .ok_or(format!("Output {} missing 'script_pubkey' (string)", idx))?;

        let script_pubkey = hex_to_bytes(script_pubkey_hex)?;

        outputs.push(TxOutput {
            value,
            script_pubkey,
        });
    }

    Ok(outputs)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    eprintln!("Parsing transaction inputs...");
    let inputs = parse_inputs(&args.inputs)?;
    eprintln!("✓ Parsed {} input(s)", inputs.len());

    eprintln!("Parsing transaction outputs...");
    let outputs = parse_outputs(&args.outputs)?;
    eprintln!("✓ Parsed {} output(s)", outputs.len());

    let trx = Transaction {
        version: args.version,
        inputs,
        outputs,
        locktime: args.locktime,
        segwit: args.segwit,
    };

    eprintln!("Serializing transaction...");
    let serialized = serialize_transaction(&trx);

    println!("\n=== TRANSACTION SERIALIZATION RESULT ===\n");
    println!("Serialized Hex:");
    println!("{}", bytes_to_hex(&serialized));
    println!("\nTransaction size: {} bytes", serialized.len());
    println!("\nTransaction details:");
    println!("  Version: {}", trx.version);
    println!("  SegWit: {}", trx.segwit);
    println!("  Inputs: {}", trx.inputs.len());
    println!("  Outputs: {}", trx.outputs.len());
    println!("  Locktime: {}", trx.locktime);

    Ok(())
}   

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

fn serialize_transaction(trx: &Transaction) -> Vec<u8> {
    let mut result = Vec::new();

    result.extend_from_slice(&trx.version.to_le_bytes());

    if trx.segwit {
        result.push(0x00);
        result.push(0x01);
    };

    result.extend_from_slice(&encode_varint(trx.inputs.len()));

    for input in &trx.inputs {
        result.extend_from_slice(&input.prev_txid);
        result.extend_from_slice(&input.vout.to_le_bytes());
        result.extend_from_slice(&encode_varint(input.script_sig.len()));
        result.extend_from_slice(&input.script_sig);
        result.extend_from_slice(&input.sequence.to_le_bytes());
    }

    result.extend_from_slice(&encode_varint(trx.outputs.len()));

    for output in &trx.outputs {
        result.extend_from_slice(&output.value.to_le_bytes());
        result.extend_from_slice(&encode_varint(output.script_pubkey.len()));
        result.extend_from_slice(&output.script_pubkey);
    }

    if trx.segwit {
        for input in &trx.inputs {
            result.extend_from_slice(&encode_varint(input.witness.len()));

            for item in &input.witness {
                result.extend_from_slice(&encode_varint(item.len()));
                result.extend_from_slice(item);
            }
        }
    }

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