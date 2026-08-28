use std::env;
use std::error::Error;
use std::fmt;

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

#[derive(Debug)]
struct CliError(String);

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for CliError {}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() || args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_usage();
        return Ok(());
    }

    let transaction = parse_args(&args)?;

    let serialized = serialize_transaction(&transaction);

    println!("Serialized transaction:");
    println!("{}", bytes_to_hex(&serialized));
    println!();
    println!("Transaction size: {} bytes", serialized.len());

    Ok(())
}

fn parse_args(args: &[String]) -> Result<Transaction, Box<dyn Error>> {
    let mut version: Option<i32> = None;
    let mut segwit: Option<bool> = None;
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    let mut locktime: Option<u32> = None;

    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--version" => {
                let value = next_arg(args, &mut i, "--version")?;

                version =
                    Some(value.parse::<i32>().map_err(|_| {
                        CliError("version must be a valid 32-bit integer".to_string())
                    })?);
            }

            "--segwit" => {
                let value = next_arg(args, &mut i, "--segwit")?;

                segwit = Some(parse_bool(&value)?);
            }

            "--input" => {
                let value = next_arg(args, &mut i, "--input")?;

                inputs.push(parse_input(&value)?);
            }

            "--output" => {
                let value = next_arg(args, &mut i, "--output")?;

                outputs.push(parse_output(&value)?);
            }

            "--locktime" => {
                let value = next_arg(args, &mut i, "--locktime")?;

                locktime = Some(value.parse::<u32>().map_err(|_| {
                    CliError("locktime must be a valid unsigned 32-bit integer".to_string())
                })?);
            }

            unknown => {
                return Err(CliError(format!(
                    "unknown argument: {}\n\nRun with --help to see usage.",
                    unknown
                ))
                .into());
            }
        }

        i += 1;
    }

    let version =
        version.ok_or_else(|| CliError("missing required argument: --version".to_string()))?;

    let segwit =
        segwit.ok_or_else(|| CliError("missing required argument: --segwit".to_string()))?;

    if inputs.is_empty() {
        return Err(CliError("at least one --input is required".to_string()).into());
    }

    if outputs.is_empty() {
        return Err(CliError("at least one --output is required".to_string()).into());
    }

    let locktime =
        locktime.ok_or_else(|| CliError("missing required argument: --locktime".to_string()))?;

    if !segwit && inputs.iter().any(|input| !input.witness.is_empty()) {
        return Err(CliError("witness data was provided but --segwit is false".to_string()).into());
    }

    Ok(Transaction {
        version,
        inputs,
        outputs,
        locktime,
        segwit,
    })
}

fn next_arg(args: &[String], index: &mut usize, argument: &str) -> Result<String, Box<dyn Error>> {
    *index += 1;

    if *index >= args.len() {
        return Err(CliError(format!("missing value for {}", argument)).into());
    }

    Ok(args[*index].clone())
}

fn parse_bool(value: &str) -> Result<bool, Box<dyn Error>> {
    match value.to_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(CliError(format!(
            "invalid SegWit value '{}'; use true or false",
            value
        ))
        .into()),
    }
}

// Format:
//
// TXID:VOUT:SCRIPTSIG:SEQUENCE:WITNESS1|WITNESS2|...
//
// Example:
//
// 8fb0...c821:1::4294967295:3045...|029c...
//
// scriptSig and witness can be empty.
fn parse_input(value: &str) -> Result<TxInput, Box<dyn Error>> {
    let parts: Vec<&str> = value.split(':').collect();

    if parts.len() != 5 {
        return Err(CliError(
            "invalid input format. Expected: TXID:VOUT:SCRIPTSIG:SEQUENCE:WITNESS1|WITNESS2|..."
                .to_string(),
        )
        .into());
    }

    let prev_txid = hex_to_bytes(parts[0])?;

    if prev_txid.len() != 32 {
        return Err(CliError(format!(
            "input TXID must be exactly 32 bytes (64 hex characters), got {} bytes",
            prev_txid.len()
        ))
        .into());
    }

    let vout = parts[1]
        .parse::<u32>()
        .map_err(|_| CliError(format!("invalid output index: {}", parts[1])))?;

    let script_sig = hex_to_bytes(parts[2])?;

    let sequence = parts[3]
        .parse::<u32>()
        .map_err(|_| CliError(format!("invalid sequence: {}", parts[3])))?;

    let witness = if parts[4].is_empty() {
        Vec::new()
    } else {
        parts[4]
            .split('|')
            .map(hex_to_bytes)
            .collect::<Result<Vec<_>, _>>()?
    };

    Ok(TxInput {
        prev_txid,
        vout,
        script_sig,
        sequence,
        witness,
    })
}

// Format:
//
// VALUE_IN_SATOSHIS:SCRIPTPUBKEY
//
// Example:
//
// 69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b
fn parse_output(value: &str) -> Result<TxOutput, Box<dyn Error>> {
    let parts: Vec<&str> = value.split(':').collect();

    if parts.len() != 2 {
        return Err(CliError(
            "invalid output format. Expected: VALUE_IN_SATOSHIS:SCRIPTPUBKEY".to_string(),
        )
        .into());
    }

    let amount = parts[0]
        .parse::<u64>()
        .map_err(|_| CliError(format!("invalid output value: {}", parts[0])))?;

    let script_pubkey = hex_to_bytes(parts[1])?;

    Ok(TxOutput {
        value: amount,
        script_pubkey,
    })
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if !hex.len().is_multiple_of(2) {
        return Err(CliError(format!(
            "invalid hexadecimal '{}': hex must have an even number of characters",
            hex
        ))
        .into());
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);

    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16)
            .map_err(|_| CliError(format!("invalid hexadecimal value '{}'", &hex[i..i + 2])))?;

        bytes.push(byte);
    }

    Ok(bytes)
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn serialize_transaction(trx: &Transaction) -> Vec<u8> {
    let mut result = Vec::new();

    // Version
    result.extend_from_slice(&trx.version.to_le_bytes());

    // SegWit marker and flag
    if trx.segwit {
        result.push(0x00);
        result.push(0x01);
    }

    // Input count
    result.extend_from_slice(&encode_varint(trx.inputs.len()));

    // Inputs
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

    // Outputs
    for output in &trx.outputs {
        // Value in satoshis
        result.extend_from_slice(&output.value.to_le_bytes());

        // ScriptPubKey length
        result.extend_from_slice(&encode_varint(output.script_pubkey.len()));

        // ScriptPubKey
        result.extend_from_slice(&output.script_pubkey);
    }

    // Witness data
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

    // Locktime
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

fn print_usage() {
    println!(
        r#"Bitcoin Transaction Serializer

Usage:

  cargo run -- \
    --version <VERSION> \
    --segwit <true|false> \
    --input <TXID:VOUT:SCRIPTSIG:SEQUENCE:WITNESS> \
    --output <VALUE_SATS:SCRIPTPUBKEY> \
    --locktime <LOCKTIME>

Input format:

  TXID:VOUT:SCRIPTSIG:SEQUENCE:WITNESS1|WITNESS2|...

Output format:

  VALUE_IN_SATOSHIS:SCRIPTPUBKEY

The --input and --output arguments can be provided multiple times
to create transactions with multiple inputs and outputs.

Example:

  cargo run -- \
    --version 2 \
    --segwit true \
    --input "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821:1::4294967295:3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301|029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358" \
    --output "69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b" \
    --output "29442:00149831122b93d21715c70db626ccc844d3c21f9687" \
    --locktime 0
"#
    );
}
