use std::collections::HashMap;
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

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if hex.len() % 2 != 0 {
        return Err(format!("hex string '{hex}' must have an even number of characters").into());
    }

    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("hex string '{hex}' contains a non-hexadecimal character").into());
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

// ---------------------------------------------------------------------
// Command line interface
//
// Rather than hardcoding a transaction in source, every value needed to
// build one is supplied on the command line. Inputs, outputs and witness
// items can all repeat (clap collects repeated `--input`/`--output`/
// `--witness` flags into a Vec), which is how multiple inputs/outputs are
// represented. Each occurrence is a small `key=value,key=value` spec
// string so a single flag can carry several fields without needing a
// separate CLI flag per field.
// ---------------------------------------------------------------------

const INPUT_HELP: &str = "A transaction input. Repeat this flag once per input.\n\
Format: txid=<64 hex chars>,vout=<number>[,sequence=<number>][,script-sig=<hex>]\n\
  txid        the previous transaction's txid, as 32 bytes of hex (required)\n\
  vout        the output index being spent (required)\n\
  sequence    input sequence number (default: 4294967295)\n\
  script-sig  scriptSig, as hex (default: empty, i.e. a SegWit input)\n\
Example: --input txid=8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821,vout=1";

const OUTPUT_HELP: &str = "A transaction output. Repeat this flag once per output.\n\
Format: value=<satoshis>,script-pubkey=<hex>\n\
Example: --output value=69886,script-pubkey=0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b";

const WITNESS_HELP: &str = "A witness stack item, attached to one input. Repeat this flag \
(in order) to build up that input's witness stack, and once per input that needs one.\n\
Format: index=<input index, 0-based>,item=<hex>\n\
Example: --witness index=0,item=3045022100f8...01";

/// Construct and serialize a Bitcoin transaction from command-line arguments.
#[derive(Debug, Parser)]
#[command(
    name = "serializetrx",
    about = "Construct and serialize a Bitcoin transaction from command-line arguments",
    after_help = "EXAMPLE\n  cargo run -- \\\n    --version 2 --locktime 0 --segwit \\\n    --input txid=8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821,vout=1 \\\n    --output value=69886,script-pubkey=0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b \\\n    --output value=29442,script-pubkey=00149831122b93d21715c70db626ccc844d3c21f9687 \\\n    --witness index=0,item=3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301 \\\n    --witness index=0,item=029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358"
)]
struct Cli {
    /// Transaction version.
    #[arg(long, default_value_t = 2)]
    version: i32,

    /// Transaction locktime.
    #[arg(long, default_value_t = 0)]
    locktime: u32,

    /// Mark this as a SegWit transaction (adds the marker/flag bytes and
    /// serializes the witness data). Omit for a legacy transaction.
    #[arg(long)]
    segwit: bool,

    /// See INPUT_HELP above.
    #[arg(long = "input", value_name = "SPEC", required = true, help = INPUT_HELP)]
    inputs: Vec<String>,

    /// See OUTPUT_HELP above.
    #[arg(long = "output", value_name = "SPEC", required = true, help = OUTPUT_HELP)]
    outputs: Vec<String>,

    /// See WITNESS_HELP above.
    #[arg(long = "witness", value_name = "SPEC", help = WITNESS_HELP)]
    witness: Vec<String>,
}

/// Parse a `key=value,key=value` spec string into a map, rejecting anything
/// that isn't a well-formed key/value pair.
fn parse_kv(kind: &str, raw: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let mut fields = HashMap::new();

    for segment in raw.split(',') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }

        let (key, value) = segment.split_once('=').ok_or_else(|| {
            format!(
                "invalid {kind} spec '{raw}': expected 'key=value' pairs separated by commas, \
                 but got '{segment}'"
            )
        })?;
        let (key, value) = (key.trim(), value.trim());

        if key.is_empty() {
            return Err(format!("invalid {kind} spec '{raw}': empty field name in '{segment}'").into());
        }
        if fields.insert(key.to_string(), value.to_string()).is_some() {
            return Err(format!("invalid {kind} spec '{raw}': field '{key}' is set more than once").into());
        }
    }

    Ok(fields)
}

/// Make sure a spec only used field names we understand, so a typo like
/// `scritp-sig=` fails loudly instead of silently falling back to a default.
fn check_known_fields(
    kind: &str,
    raw: &str,
    fields: &HashMap<String, String>,
    allowed: &[&str],
) -> Result<(), Box<dyn Error>> {
    for key in fields.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(format!(
                "invalid {kind} spec '{raw}': unknown field '{key}' (expected one of: {})",
                allowed.join(", ")
            )
            .into());
        }
    }
    Ok(())
}

fn required_field<'a>(
    kind: &str,
    raw: &str,
    fields: &'a HashMap<String, String>,
    key: &str,
) -> Result<&'a str, Box<dyn Error>> {
    fields
        .get(key)
        .map(|s| s.as_str())
        .ok_or_else(|| format!("invalid {kind} spec '{raw}': missing required field '{key}'").into())
}

fn parse_number<T: std::str::FromStr>(
    kind: &str,
    raw: &str,
    key: &str,
    value: &str,
) -> Result<T, Box<dyn Error>>
where
    T::Err: std::fmt::Display,
{
    value
        .parse::<T>()
        .map_err(|e| format!("invalid {kind} spec '{raw}': field '{key}' = '{value}' is not a valid number ({e})").into())
}

fn parse_input_spec(raw: &str) -> Result<TxInput, Box<dyn Error>> {
    let fields = parse_kv("input", raw)?;
    check_known_fields("input", raw, &fields, &["txid", "vout", "sequence", "script-sig"])?;

    let txid_hex = required_field("input", raw, &fields, "txid")?;
    let prev_txid = hex_to_bytes(txid_hex)
        .map_err(|e| format!("invalid input spec '{raw}': field 'txid': {e}"))?;
    if prev_txid.len() != 32 {
        return Err(format!(
            "invalid input spec '{raw}': field 'txid' must be exactly 32 bytes (64 hex \
             characters), got {} bytes",
            prev_txid.len()
        )
        .into());
    }

    let vout = parse_number("input", raw, "vout", required_field("input", raw, &fields, "vout")?)?;

    let sequence = match fields.get("sequence") {
        Some(v) => parse_number("input", raw, "sequence", v)?,
        None => 0xffff_ffff,
    };

    let script_sig = match fields.get("script-sig") {
        Some(v) => hex_to_bytes(v).map_err(|e| format!("invalid input spec '{raw}': field 'script-sig': {e}"))?,
        None => Vec::new(),
    };

    Ok(TxInput {
        prev_txid,
        vout,
        script_sig,
        sequence,
        witness: Vec::new(),
    })
}

fn parse_output_spec(raw: &str) -> Result<TxOutput, Box<dyn Error>> {
    let fields = parse_kv("output", raw)?;
    check_known_fields("output", raw, &fields, &["value", "script-pubkey"])?;

    let value = parse_number("output", raw, "value", required_field("output", raw, &fields, "value")?)?;

    let script_pubkey_hex = required_field("output", raw, &fields, "script-pubkey")?;
    let script_pubkey = hex_to_bytes(script_pubkey_hex)
        .map_err(|e| format!("invalid output spec '{raw}': field 'script-pubkey': {e}"))?;

    Ok(TxOutput { value, script_pubkey })
}

/// Returns (input index, witness item bytes).
fn parse_witness_spec(raw: &str) -> Result<(usize, Vec<u8>), Box<dyn Error>> {
    let fields = parse_kv("witness", raw)?;
    check_known_fields("witness", raw, &fields, &["index", "item"])?;

    let index = parse_number("witness", raw, "index", required_field("witness", raw, &fields, "index")?)?;

    let item_hex = required_field("witness", raw, &fields, "item")?;
    let item = hex_to_bytes(item_hex).map_err(|e| format!("invalid witness spec '{raw}': field 'item': {e}"))?;

    Ok((index, item))
}

fn build_transaction(cli: &Cli) -> Result<Transaction, Box<dyn Error>> {
    let mut inputs = cli
        .inputs
        .iter()
        .map(|spec| parse_input_spec(spec))
        .collect::<Result<Vec<_>, _>>()?;

    let outputs = cli
        .outputs
        .iter()
        .map(|spec| parse_output_spec(spec))
        .collect::<Result<Vec<_>, _>>()?;

    for spec in &cli.witness {
        let (index, item) = parse_witness_spec(spec)?;
        let input_count = inputs.len();
        let input = inputs.get_mut(index).ok_or_else(|| {
            format!("invalid witness spec '{spec}': index {index} is out of range ({input_count} input(s) provided)")
        })?;
        input.witness.push(item);
    }

    Ok(Transaction {
        version: cli.version,
        inputs,
        outputs,
        locktime: cli.locktime,
        segwit: cli.segwit,
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let trx = build_transaction(&cli)?;

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
