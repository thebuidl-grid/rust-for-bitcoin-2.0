use std::error::Error;
use clap::Parser;

// ─────────────────────────────────────────────────────────────────────────────
// Data structures
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// CLI definition
// ─────────────────────────────────────────────────────────────────────────────

/// Serialize a Bitcoin transaction from command-line arguments.
///
/// Multiple inputs and outputs are supported by repeating --input / --output.
/// Witness data is supplied with --witness, one per input, in the same order
/// as the --input flags.
#[derive(Parser, Debug)]
#[command(name = "serializetrx")]
#[command(about = "Build and serialize a Bitcoin transaction from CLI arguments")]
#[command(
    after_help = "\
FORMATS
  --input   TXID:VOUT:SEQUENCE:SCRIPTSIG_HEX
              TXID          – 64-char hex (32 bytes), will be reversed to
                              internal byte order
              VOUT          – decimal output index (u32)
              SEQUENCE      – decimal or hex (0x…) sequence number (u32)
              SCRIPTSIG_HEX – hex-encoded scriptSig bytes, or empty string \"\"
                              for native SegWit inputs

  --output  VALUE_SATS:SCRIPTPUBKEY_HEX
              VALUE_SATS    – output value in satoshis (u64)
              SCRIPTPUBKEY_HEX – hex-encoded scriptPubKey bytes

  --witness ITEM1_HEX,ITEM2_HEX,...
              One flag per input (in the same order as --input).
              Witness items are comma-separated hex strings.
              For inputs with no witness data use an empty string \"\".

EXAMPLES
  # Legacy (non-SegWit) transaction
  serializetrx \\
    --version 1 \\
    --input 8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821:0:4294967295:76a91489abcdefabbaabbaabbaabbaabbaabbaabbaabba88ac \\
    --output 50000:76a91489abcdefabbaabbaabbaabbaabbaabbaabbaabba88ac \\
    --locktime 0

  # Native SegWit (P2WPKH) transaction
  serializetrx \\
    --version 2 \\
    --segwit \\
    --input 8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821:1:4294967295: \\
    --witness 3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301,029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358 \\
    --output 69886:0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b \\
    --output 29442:00149831122b93d21715c70db626ccc844d3c21f9687 \\
    --locktime 0\
"
)]
struct Cli {
    /// Transaction version (default: 2)
    #[arg(long, default_value_t = 2)]
    version: i32,

    /// Mark transaction as SegWit (adds marker/flag bytes and serializes witness)
    #[arg(long, default_value_t = false)]
    segwit: bool,

    /// One or more inputs.  Format: TXID:VOUT:SEQUENCE:SCRIPTSIG_HEX
    /// Repeat the flag for multiple inputs.
    #[arg(long = "input", value_name = "INPUT", required = true)]
    inputs: Vec<String>,

    /// One or more outputs.  Format: VALUE_SATS:SCRIPTPUBKEY_HEX
    /// Repeat the flag for multiple outputs.
    #[arg(long = "output", value_name = "OUTPUT", required = true)]
    outputs: Vec<String>,

    /// Witness data for each input (one --witness per input, in input order).
    /// Items are comma-separated hex strings.  Use \"\" for inputs without witness.
    #[arg(long = "witness", value_name = "WITNESS")]
    witnesses: Vec<String>,

    /// Locktime (default: 0)
    #[arg(long, default_value_t = 0)]
    locktime: u32,
}

// ─────────────────────────────────────────────────────────────────────────────
// Hex helpers
// ─────────────────────────────────────────────────────────────────────────────

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let hex = hex.trim();
    if hex.len() % 2 != 0 {
        return Err(format!(
            "hex string has odd length ({}): \"{}\"",
            hex.len(),
            hex
        )
        .into());
    }
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| {
            format!(
                "invalid hex character at position {}: \"{}\"",
                i,
                &hex[i..i + 2]
            )
        })?;
        bytes.push(byte);
    }
    Ok(bytes)
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// Parsing / validation helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Parse a decimal or 0x-prefixed hex u32.
fn parse_u32(s: &str) -> Result<u32, Box<dyn Error>> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(hex, 16)
            .map_err(|e| format!("invalid hex u32 \"{}\": {}", s, e).into())
    } else {
        s.parse::<u32>()
            .map_err(|e| format!("invalid decimal u32 \"{}\": {}", s, e).into())
    }
}

/// Parse a TXID hex string (64 chars) and reverse it into internal byte order.
fn parse_txid(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let hex = hex.trim();
    if hex.len() != 64 {
        return Err(format!(
            "TXID must be 64 hex characters (32 bytes), got {} characters: \"{}\"",
            hex.len(),
            hex
        )
        .into());
    }
    let mut bytes = hex_to_bytes(hex)?;
    bytes.reverse(); // display order → internal (little-endian) byte order
    Ok(bytes)
}

/// Parse one `--input` string: `TXID:VOUT:SEQUENCE:SCRIPTSIG_HEX`
fn parse_input(raw: &str) -> Result<(Vec<u8>, u32, u32, Vec<u8>), Box<dyn Error>> {
    // Split on ':' but allow the TXID (which contains no ':') to be first.
    // We always expect exactly 4 colon-delimited fields.
    let parts: Vec<&str> = raw.splitn(4, ':').collect();
    if parts.len() != 4 {
        return Err(format!(
            "input must have the format TXID:VOUT:SEQUENCE:SCRIPTSIG_HEX, got: \"{}\"",
            raw
        )
        .into());
    }

    let txid = parse_txid(parts[0])
        .map_err(|e| format!("input TXID error: {}", e))?;

    let vout = parts[1]
        .trim()
        .parse::<u32>()
        .map_err(|e| format!("input VOUT \"{}\" is not a valid u32: {}", parts[1].trim(), e))?;

    let sequence = parse_u32(parts[2])
        .map_err(|e| format!("input SEQUENCE error: {}", e))?;

    let script_sig = if parts[3].trim().is_empty() {
        vec![]
    } else {
        hex_to_bytes(parts[3].trim())
            .map_err(|e| format!("input SCRIPTSIG_HEX error: {}", e))?
    };

    Ok((txid, vout, sequence, script_sig))
}

/// Parse one `--output` string: `VALUE_SATS:SCRIPTPUBKEY_HEX`
fn parse_output(raw: &str) -> Result<(u64, Vec<u8>), Box<dyn Error>> {
    let parts: Vec<&str> = raw.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(format!(
            "output must have the format VALUE_SATS:SCRIPTPUBKEY_HEX, got: \"{}\"",
            raw
        )
        .into());
    }

    let value = parts[0]
        .trim()
        .parse::<u64>()
        .map_err(|e| format!("output VALUE \"{}\" is not a valid u64: {}", parts[0].trim(), e))?;

    let script_pubkey = hex_to_bytes(parts[1].trim())
        .map_err(|e| format!("output SCRIPTPUBKEY_HEX error: {}", e))?;

    Ok((value, script_pubkey))
}

/// Parse one `--witness` string: comma-separated hex items.
/// An empty string means no witness items for that input.
fn parse_witness(raw: &str) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Ok(vec![]);
    }
    raw.split(',')
        .map(|item| {
            let item = item.trim();
            if item.is_empty() {
                Ok(vec![])
            } else {
                hex_to_bytes(item).map_err(|e| format!("witness item error: {}", e).into())
            }
        })
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// Serialization
// ─────────────────────────────────────────────────────────────────────────────

fn encode_varint(value: usize) -> Vec<u8> {
    match value {
        0..=0xfc => vec![value as u8],
        0xfd..=0xffff => {
            let mut r = vec![0xfd];
            r.extend_from_slice(&(value as u16).to_le_bytes());
            r
        }
        0x10000..=0xffff_ffff => {
            let mut r = vec![0xfe];
            r.extend_from_slice(&(value as u32).to_le_bytes());
            r
        }
        _ => {
            let mut r = vec![0xff];
            r.extend_from_slice(&(value as u64).to_le_bytes());
            r
        }
    }
}

/// Serialize a Bitcoin transaction following the BIP-141 extended format for
/// SegWit or the legacy format for non-SegWit transactions.
///
/// Layout (SegWit):
/// ┌──────────────────────────────┐
/// │ Version          4 bytes     │
/// ├──────────────────────────────┤
/// │ Marker           1 byte      │  (0x00)
/// │ Flag             1 byte      │  (0x01)
/// ├──────────────────────────────┤
/// │ Input count      VarInt      │
/// │ Inputs           Variable    │
/// ├──────────────────────────────┤
/// │ Output count     VarInt      │
/// │ Outputs          Variable    │
/// ├──────────────────────────────┤
/// │ Witness          Variable    │
/// ├──────────────────────────────┤
/// │ Locktime         4 bytes     │
/// └──────────────────────────────┘
fn serialize_transaction(trx: &Transaction) -> Vec<u8> {
    let mut result = Vec::new();

    // Version (4 bytes, little-endian)
    result.extend_from_slice(&trx.version.to_le_bytes());

    // SegWit marker + flag
    if trx.segwit {
        result.push(0x00); // marker
        result.push(0x01); // flag
    }

    // Input count
    result.extend_from_slice(&encode_varint(trx.inputs.len()));

    // Inputs
    for input in &trx.inputs {
        result.extend_from_slice(&input.prev_txid);
        result.extend_from_slice(&input.vout.to_le_bytes());
        result.extend_from_slice(&encode_varint(input.script_sig.len()));
        result.extend_from_slice(&input.script_sig);
        result.extend_from_slice(&input.sequence.to_le_bytes());
    }

    // Output count
    result.extend_from_slice(&encode_varint(trx.outputs.len()));

    // Outputs
    for output in &trx.outputs {
        result.extend_from_slice(&output.value.to_le_bytes());
        result.extend_from_slice(&encode_varint(output.script_pubkey.len()));
        result.extend_from_slice(&output.script_pubkey);
    }

    // Witness data (one stack per input)
    if trx.segwit {
        for input in &trx.inputs {
            result.extend_from_slice(&encode_varint(input.witness.len()));
            for item in &input.witness {
                result.extend_from_slice(&encode_varint(item.len()));
                result.extend_from_slice(item);
            }
        }
    }

    // Locktime (4 bytes, little-endian)
    result.extend_from_slice(&trx.locktime.to_le_bytes());

    result
}

// ─────────────────────────────────────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────────────────────────────────────

fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // ── Validate witness count ────────────────────────────────────────────────
    // If any --witness flags are provided, there must be exactly one per input.
    if !cli.witnesses.is_empty() && cli.witnesses.len() != cli.inputs.len() {
        return Err(format!(
            "--witness count ({}) must match --input count ({}). \
             Provide one --witness per input; use \"\" for inputs with no witness.",
            cli.witnesses.len(),
            cli.inputs.len()
        )
        .into());
    }

    // ── Parse inputs ─────────────────────────────────────────────────────────
    let mut parsed_inputs: Vec<TxInput> = Vec::with_capacity(cli.inputs.len());

    for (idx, raw_input) in cli.inputs.iter().enumerate() {
        let (txid, vout, sequence, script_sig) =
            parse_input(raw_input).map_err(|e| format!("--input[{}]: {}", idx, e))?;

        let witness = if cli.witnesses.is_empty() {
            vec![]
        } else {
            parse_witness(&cli.witnesses[idx])
                .map_err(|e| format!("--witness[{}]: {}", idx, e))?
        };

        parsed_inputs.push(TxInput {
            prev_txid: txid,
            vout,
            script_sig,
            sequence,
            witness,
        });
    }

    // ── Parse outputs ─────────────────────────────────────────────────────────
    let mut parsed_outputs: Vec<TxOutput> = Vec::with_capacity(cli.outputs.len());

    for (idx, raw_output) in cli.outputs.iter().enumerate() {
        let (value, script_pubkey) =
            parse_output(raw_output).map_err(|e| format!("--output[{}]: {}", idx, e))?;

        parsed_outputs.push(TxOutput {
            value,
            script_pubkey,
        });
    }

    // ── Build transaction ─────────────────────────────────────────────────────
    let trx = Transaction {
        version: cli.version,
        inputs: parsed_inputs,
        outputs: parsed_outputs,
        locktime: cli.locktime,
        segwit: cli.segwit,
    };

    // ── Serialize & display ───────────────────────────────────────────────────
    let serialized = serialize_transaction(&trx);

    println!("Serialized hex:");
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
