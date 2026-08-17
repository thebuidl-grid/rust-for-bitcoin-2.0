use std::error::Error;
use clap::Parser;

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
#[command(
    name = "serialize_trx",
    version = "0.1.0",
    about = "Bitcoin transaction serializer CLI",
    disable_version_flag = true
)]
struct Cli {
    /// Transaction version
    #[arg(long, short = 'v', default_value = "2")]
    version: i32,

    /// Enable SegWit status (required for witness serialization)
    #[arg(long, short = 's')]
    segwit: bool,

    /// Locktime (4-byte little-endian integer)
    #[arg(long, short = 'l', default_value = "0")]
    locktime: u32,

    /// Multiple transaction inputs.
    /// Format: prev_txid=<hex>,vout=<u32>[,sequence=<u32>][,script_sig=<hex>][,witness=<hex1>:<hex2>:...]
    #[arg(long = "input", short = 'i', required = true)]
    inputs: Vec<String>,

    /// Multiple transaction outputs.
    /// Format: value=<u64>,script_pubkey=<hex>
    #[arg(long = "output", short = 'o', required = true)]
    outputs: Vec<String>,
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let cleaned = hex.trim();
    if cleaned.len() % 2 != 0 {
        return Err("Hex string must have an even length".into());
    }

    let mut bytes = Vec::with_capacity(cleaned.len() / 2);
    for i in (0..cleaned.len()).step_by(2) {
        let byte = u8::from_str_radix(&cleaned[i..i + 2], 16)
            .map_err(|e| format!("Invalid hex character sequence '{}': {}", &cleaned[i..i + 2], e))?;
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

fn parse_input(s: &str) -> Result<TxInput, Box<dyn Error>> {
    let mut prev_txid = None;
    let mut vout = None;
    let mut sequence = Some(0xffffffff);
    let mut script_sig = Some(Vec::new());
    let mut witness = Some(Vec::new());

    for part in s.split(',') {
        let mut kv = part.splitn(2, '=');
        let key = kv.next().ok_or("Empty key-value pair in input specification")?.trim();
        let value = kv.next().ok_or(format!("Missing '=' in input parameter '{}'", part))?.trim();

        match key {
            "prev_txid" => {
                let bytes = hex_to_bytes(value)
                    .map_err(|e| format!("Invalid prev_txid: {}", e))?;
                if bytes.len() != 32 {
                    return Err(format!("prev_txid must be exactly 32 bytes (64 hex characters), got {} bytes", bytes.len()).into());
                }
                prev_txid = Some(bytes);
            }
            "vout" => {
                let v = value.parse::<u32>()
                    .map_err(|e| format!("Invalid vout index '{}': {}", value, e))?;
                vout = Some(v);
            }
            "sequence" => {
                let seq = value.parse::<u32>()
                    .map_err(|e| format!("Invalid sequence '{}': {}", value, e))?;
                sequence = Some(seq);
            }
            "script_sig" => {
                let bytes = hex_to_bytes(value)
                    .map_err(|e| format!("Invalid script_sig: {}", e))?;
                script_sig = Some(bytes);
            }
            "witness" => {
                let mut items = Vec::new();
                if !value.is_empty() {
                    for item_str in value.split(':') {
                        let bytes = hex_to_bytes(item_str)
                            .map_err(|e| format!("Invalid witness item hex: {}", e))?;
                        items.push(bytes);
                    }
                }
                witness = Some(items);
            }
            _ => return Err(format!("Unknown input key '{}'", key).into()),
        }
    }

    let prev_txid = prev_txid.ok_or("Missing required field 'prev_txid' in input")?;
    let vout = vout.ok_or("Missing required field 'vout' in input")?;

    Ok(TxInput {
        prev_txid,
        vout,
        script_sig: script_sig.unwrap(),
        sequence: sequence.unwrap(),
        witness: witness.unwrap(),
    })
}

fn parse_output(s: &str) -> Result<TxOutput, Box<dyn Error>> {
    let mut value = None;
    let mut script_pubkey = None;

    for part in s.split(',') {
        let mut kv = part.splitn(2, '=');
        let key = kv.next().ok_or("Empty key-value pair in output specification")?.trim();
        let value_str = kv.next().ok_or(format!("Missing '=' in output parameter '{}'", part))?.trim();

        match key {
            "value" | "amount" => {
                let val = value_str.parse::<u64>()
                    .map_err(|e| format!("Invalid output value '{}': {}", value_str, e))?;
                value = Some(val);
            }
            "script_pubkey" => {
                let bytes = hex_to_bytes(value_str)
                    .map_err(|e| format!("Invalid script_pubkey: {}", e))?;
                script_pubkey = Some(bytes);
            }
            _ => return Err(format!("Unknown output key '{}'", key).into()),
        }
    }

    let value = value.ok_or("Missing required field 'value' in output")?;
    let script_pubkey = script_pubkey.ok_or("Missing required field 'script_pubkey' in output")?;

    Ok(TxOutput {
        value,
        script_pubkey,
    })
}

fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let mut inputs = Vec::new();
    for (idx, input_str) in cli.inputs.iter().enumerate() {
        let input = parse_input(input_str)
            .map_err(|e| format!("Error in input {}: {}", idx, e))?;
        inputs.push(input);
    }

    let mut outputs = Vec::new();
    for (idx, output_str) in cli.outputs.iter().enumerate() {
        let output = parse_output(output_str)
            .map_err(|e| format!("Error in output {}: {}", idx, e))?;
        outputs.push(output);
    }

    // Validate that if SegWit is false, no inputs have witness items
    if !cli.segwit {
        for (idx, input) in inputs.iter().enumerate() {
            if !input.witness.is_empty() {
                return Err(format!(
                    "Validation Error: Input {} contains witness items but SegWit status (--segwit) is not enabled.",
                    idx
                ).into());
            }
        }
    }

    let trx = Transaction {
        version: cli.version,
        inputs,
        outputs,
        locktime: cli.locktime,
        segwit: cli.segwit,
    };

    let serialized = serialize_transaction(&trx);

    println!("Serialized transaction:");
    println!("{:?}", &serialized);
    println!("Serialized Hex transaction:");
    println!("{}", bytes_to_hex(&serialized));
    println!("\nTransaction size: {} bytes", serialized.len());

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn serialize_transaction(trx: &Transaction) -> Vec<u8> {
    let mut result = Vec::new();

    // add version number
    result.extend_from_slice(&trx.version.to_le_bytes());

    if trx.segwit {
        result.push(0x00); // marker
        result.push(0x01); // flag
    }

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