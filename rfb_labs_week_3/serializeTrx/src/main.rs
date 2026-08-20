use clap::Parser;

// === Data structures

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

// === CLI

/// Serialize a Bitcoin transaction from command-line arguments.
///
/// Each --input flag describes one transaction input.
/// Each --output flag describes one transaction output.
/// Each --witness flag describes the witness stack for one input (SegWit only).
///
/// INPUT format:  TXID_HEX:VOUT:SEQUENCE_HEX:SCRIPTSIG_HEX
///   TXID_HEX      - 64-char hex string (32 bytes, as shown on block explorers)
///   VOUT          - output index as a decimal integer
///   SEQUENCE_HEX  - 4-byte sequence as hex (e.g. ffffffff)
///   SCRIPTSIG_HEX - hex-encoded scriptSig bytes, or empty for SegWit inputs
///
/// OUTPUT format: VALUE_SATS:SCRIPTPUBKEY_HEX
///   VALUE_SATS      - amount in satoshis as a decimal integer
///   SCRIPTPUBKEY_HEX - hex-encoded scriptPubKey bytes
///
/// WITNESS format: ITEM1_HEX,ITEM2_HEX,...
///   Comma-separated hex items for one input's witness stack.
///   Provide one --witness per input, in the same order as --input.
///   Inputs with no witness data use an empty string: --witness ""
#[derive(Parser, Debug)]
#[command(name = "serializetrx")]
#[command(about = "Construct and serialize a Bitcoin transaction")]
#[command(version = "1.0")]
struct Cli {
    /// Transaction version (e.g. 1 or 2)
    #[arg(long = "tx-version", default_value = "2")]
    tx_version: i32,

    /// Enable SegWit serialization (adds marker/flag bytes and witness data)
    #[arg(long)]
    segwit: bool,

    /// Transaction input: TXID_HEX:VOUT:SEQUENCE_HEX:SCRIPTSIG_HEX
    #[arg(long = "input", required = true)]
    inputs: Vec<String>,

    /// Transaction output: VALUE_SATS:SCRIPTPUBKEY_HEX
    #[arg(long = "output", required = true)]
    outputs: Vec<String>,

    /// Witness stack for one input (comma-separated hex items).
    /// Supply one --witness per input in input order.
    /// Use --witness "" for inputs with no witness.
    #[arg(long = "witness")]
    witnesses: Vec<String>,

    /// Locktime as a decimal integer
    #[arg(long, default_value = "0")]
    locktime: u32,
}

// === Validation helpers

/// Validate and decode a hex string, returning a descriptive error.
fn decode_hex(value: &str, field: &str) -> Result<Vec<u8>, String> {
    if value.len() % 2 != 0 {
        return Err(format!(
            "{field}: hex string has odd length ({}); each byte needs two hex characters",
            value.len()
        ));
    }
    hex::decode(value).map_err(|e| format!("{field}: invalid hex '{value}': {e}"))
}

/// Parse and validate a transaction input from its string representation.
///
/// Format: TXID_HEX:VOUT:SEQUENCE_HEX:SCRIPTSIG_HEX
/// SCRIPTSIG_HEX may be empty (native SegWit inputs have no scriptSig).
fn parse_input(raw: &str, index: usize) -> Result<TxInput, String> {
    let label = format!("input[{index}]");

    // Split on ':' but allow at most 4 parts so an empty scriptSig is handled.
    let parts: Vec<&str> = raw.splitn(4, ':').collect();
    if parts.len() != 4 {
        return Err(format!(
            "{label}: expected format TXID_HEX:VOUT:SEQUENCE_HEX:SCRIPTSIG_HEX, got '{raw}'"
        ));
    }

    let (txid_str, vout_str, seq_str, script_str) = (parts[0], parts[1], parts[2], parts[3]);

    // TXID: must be 64 hex chars (32 bytes). Block explorers show TXIDs
    // byte-reversed, so we reverse here to get internal byte order.
    if txid_str.len() != 64 {
        return Err(format!(
            "{label}: TXID must be 64 hex characters (32 bytes), got {} chars",
            txid_str.len()
        ));
    }
    let mut txid_bytes = decode_hex(txid_str, &format!("{label}.txid"))?;
    txid_bytes.reverse(); // display order -> internal (little-endian) order

    // VOUT: decimal integer
    let vout: u32 = vout_str.parse().map_err(|_| {
        format!("{label}: vout must be a decimal integer, got '{vout_str}'")
    })?;

    // Sequence: 4-byte hex (e.g. "ffffffff")
    let seq_bytes = decode_hex(seq_str, &format!("{label}.sequence"))?;
    if seq_bytes.len() != 4 {
        return Err(format!(
            "{label}: sequence must be exactly 4 bytes (8 hex chars), got {}",
            seq_bytes.len()
        ));
    }
    let sequence = u32::from_le_bytes(seq_bytes.try_into().unwrap());

    // ScriptSig: may be empty
    let script_sig = if script_str.is_empty() {
        vec![]
    } else {
        decode_hex(script_str, &format!("{label}.script_sig"))?
    };

    Ok(TxInput {
        prev_txid: txid_bytes,
        vout,
        script_sig,
        sequence,
        witness: vec![], // populated separately from --witness flags
    })
}

/// Parse and validate a transaction output from its string representation.
///
/// Format: VALUE_SATS:SCRIPTPUBKEY_HEX
fn parse_output(raw: &str, index: usize) -> Result<TxOutput, String> {
    let label = format!("output[{index}]");

    let parts: Vec<&str> = raw.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(format!(
            "{label}: expected format VALUE_SATS:SCRIPTPUBKEY_HEX, got '{raw}'"
        ));
    }

    let (value_str, script_str) = (parts[0], parts[1]);

    let value: u64 = value_str.parse().map_err(|_| {
        format!("{label}: value must be a decimal integer (satoshis), got '{value_str}'")
    })?;

    let script_pubkey = decode_hex(script_str, &format!("{label}.script_pubkey"))?;

    Ok(TxOutput { value, script_pubkey })
}

/// Parse and validate a witness stack from its string representation.
///
/// Format: ITEM1_HEX,ITEM2_HEX,...  (empty string means no witness items)
fn parse_witness(raw: &str, input_index: usize) -> Result<Vec<Vec<u8>>, String> {
    if raw.is_empty() {
        return Ok(vec![]);
    }
    raw.split(',')
        .enumerate()
        .map(|(item_index, item_hex)| {
            decode_hex(
                item_hex.trim(),
                &format!("witness[{input_index}][{item_index}]"),
            )
        })
        .collect()
}

// === Serialization (unchanged logic from original program)

fn serialize_transaction(trx: &Transaction) -> Vec<u8> {
    let mut result = Vec::new();

    result.extend_from_slice(&trx.version.to_le_bytes());

    if trx.segwit {
        result.push(0x00); // marker
        result.push(0x01); // flag
    }

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
            let mut v = vec![0xfd];
            v.extend_from_slice(&(value as u16).to_le_bytes());
            v
        }
        0x10000..=0xffff_ffff => {
            let mut v = vec![0xfe];
            v.extend_from_slice(&(value as u32).to_le_bytes());
            v
        }
        _ => {
            let mut v = vec![0xff];
            v.extend_from_slice(&(value as u64).to_le_bytes());
            v
        }
    }
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

// === Entry point

fn main() {
    let cli = Cli::parse();

    // Validate SegWit + witness consistency.
    // When --segwit is set, exactly one --witness entry must be present per input.
    // When --segwit is not set, --witness flags are not allowed.
    if cli.segwit && !cli.witnesses.is_empty() && cli.witnesses.len() != cli.inputs.len() {
        eprintln!(
            "Error: --segwit requires one --witness entry per input ({} inputs, {} witness flags supplied)",
            cli.inputs.len(),
            cli.witnesses.len()
        );
        std::process::exit(1);
    }
    if !cli.segwit && !cli.witnesses.is_empty() {
        eprintln!("Error: --witness flags supplied but --segwit was not set");
        std::process::exit(1);
    }

    // Parse inputs
    let mut inputs: Vec<TxInput> = match cli
        .inputs
        .iter()
        .enumerate()
        .map(|(i, raw)| parse_input(raw, i))
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    // Parse outputs
    let outputs: Vec<TxOutput> = match cli
        .outputs
        .iter()
        .enumerate()
        .map(|(i, raw)| parse_output(raw, i))
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    // Parse witnesses and attach to inputs
    if cli.segwit {
        // Pad with empty witness entries if fewer --witness flags were given than inputs
        // (allows omitting trailing empty witnesses).
        let witness_strs: Vec<&str> = if cli.witnesses.is_empty() {
            vec![""; inputs.len()]
        } else {
            cli.witnesses.iter().map(|s| s.as_str()).collect()
        };

        for (i, witness_str) in witness_strs.iter().enumerate() {
            match parse_witness(witness_str, i) {
                Ok(items) => inputs[i].witness = items,
                Err(e) => {
                    eprintln!("Error: {e}");
                    std::process::exit(1);
                }
            }
        }
    }

    let trx = Transaction {
        version: cli.tx_version,
        inputs,
        outputs,
        locktime: cli.locktime,
        segwit: cli.segwit,
    };

    let serialized = serialize_transaction(&trx);

    println!("Serialized hex:");
    println!("{}", bytes_to_hex(&serialized));
    println!("\nTransaction size: {} bytes", serialized.len());
}
