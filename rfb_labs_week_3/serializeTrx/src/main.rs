use std::error::Error;
use std::fs;
use clap::Parser;
use serde::Deserialize;

#[derive(Parser)]
#[command(name = "Transaction Serializer")]
#[command(about = "Serializes a Bitcoin transaction from a JSON file")]
struct Cli {
    /// Path to the JSON transaction file
    #[arg(short, long)]
    file: String,
}

#[derive(Debug, Deserialize)]
struct TxInput {
    prev_txid: String,
    vout: u32,
    script_sig: String,
    sequence: u32,
    witness: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TxOutput {
    value: u64,
    script_pubkey: String,
}

#[derive(Debug, Deserialize)]
struct Transaction {
    version: i32,
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
    locktime: u32,
    segwit: bool,
}

fn validate_hex(hex: &str, name: &str) -> Result<(), Box<dyn Error>> {
    if hex.is_empty() {
        return Ok(());
    }
    if hex.len() % 2 != 0 {
        return Err(format!("{}: must have even length, got {}", name, hex.len()).into());
    }
    for (i, c) in hex.chars().enumerate() {
        if !c.is_ascii_hexdigit() {
            return Err(format!("{}: invalid hex char '{}' at position {}", name, c, i).into());
        }
    }
    Ok(())
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
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
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

fn validate_transaction(trx: &Transaction) -> Result<(), Box<dyn Error>> {
    if trx.inputs.is_empty() {
        return Err("Transaction must have at least one input".into());
    }
    if trx.outputs.is_empty() {
        return Err("Transaction must have at least one output".into());
    }

    for (i, input) in trx.inputs.iter().enumerate() {
        validate_hex(&input.prev_txid, &format!("inputs[{}].prev_txid", i))?;
        if input.prev_txid.len() != 64 {
            return Err(format!(
                "inputs[{}].prev_txid: must be 32 bytes (64 hex chars), got {}",
                i, input.prev_txid.len()
            ).into());
        }
        validate_hex(&input.script_sig, &format!("inputs[{}].script_sig", i))?;
        for (j, item) in input.witness.iter().enumerate() {
            validate_hex(item, &format!("inputs[{}].witness[{}]", i, j))?;
        }
    }

    for (i, output) in trx.outputs.iter().enumerate() {
        validate_hex(&output.script_pubkey, &format!("outputs[{}].script_pubkey", i))?;
    }

    Ok(())
}

fn serialize_transaction(trx: &Transaction) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut result = Vec::new();

    result.extend_from_slice(&trx.version.to_le_bytes());

    if trx.segwit {
        result.push(0x00);
        result.push(0x01);
    }

    result.extend_from_slice(&encode_varint(trx.inputs.len()));

    for input in &trx.inputs {
        let prev_txid = hex_to_bytes(&input.prev_txid)?;
        result.extend_from_slice(&prev_txid);
        result.extend_from_slice(&input.vout.to_le_bytes());

        let script_sig = hex_to_bytes(&input.script_sig)?;
        result.extend_from_slice(&encode_varint(script_sig.len()));
        result.extend_from_slice(&script_sig);

        result.extend_from_slice(&input.sequence.to_le_bytes());
    }

    result.extend_from_slice(&encode_varint(trx.outputs.len()));

    for output in &trx.outputs {
        result.extend_from_slice(&output.value.to_le_bytes());

        let script_pubkey = hex_to_bytes(&output.script_pubkey)?;
        result.extend_from_slice(&encode_varint(script_pubkey.len()));
        result.extend_from_slice(&script_pubkey);
    }

    if trx.segwit {
        for input in &trx.inputs {
            result.extend_from_slice(&encode_varint(input.witness.len()));

            for item in &input.witness {
                let witness_bytes = hex_to_bytes(item)?;
                result.extend_from_slice(&encode_varint(witness_bytes.len()));
                result.extend_from_slice(&witness_bytes);
            }
        }
    }

    result.extend_from_slice(&trx.locktime.to_le_bytes());

    Ok(result)
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let json = fs::read_to_string(&cli.file)?;
    let trx: Transaction = serde_json::from_str(&json)?;

    validate_transaction(&trx)?;

    let serialized = serialize_transaction(&trx)?;

    println!("Serialized Hex transaction:");
    println!("{}", bytes_to_hex(&serialized));
    println!("\nTransaction size: {} bytes", serialized.len());

    Ok(())
}
