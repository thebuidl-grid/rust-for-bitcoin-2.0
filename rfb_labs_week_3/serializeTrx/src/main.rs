use clap::Parser;
use std::error::Error;
use std::fmt;
use std::io::{self, Write};

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
struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ParseError {}

fn err(msg: impl Into<String>) -> Box<dyn Error> {
    Box::new(ParseError(msg.into()))
}

/// Build and serialize a Bitcoin transaction from command-line arguments.
#[derive(Parser, Debug)]
#[command(
    name = "serializeTrx",
    version = "1.0",
    about = "Serialize a Bitcoin transaction from CLI-supplied data",
    disable_version_flag = true
)]
struct Cli {
    /// Transaction version
    #[arg(long, default_value_t = 2)]
    version: i32,

    /// Locktime
    #[arg(long, default_value_t = 0)]
    locktime: u32,

    /// Mark this as a SegWit transaction (adds marker/flag and witness data)
    #[arg(long)]
    segwit: bool,

    /// A transaction input: "<prev_txid_hex>:<vout>:<sequence>:<script_sig_hex>"
    /// script_sig_hex may be empty. Repeat this flag for multiple inputs.
    #[arg(long = "input", required = true)]
    inputs: Vec<String>,

    /// A transaction output: "<value_sats>:<script_pubkey_hex>"
    /// Repeat this flag for multiple outputs.
    #[arg(long = "output", required = true)]
    outputs: Vec<String>,

    /// Witness data for one input: "<input_index>:<hex_item_1>,<hex_item_2>,..."
    /// Repeat this flag once per input that carries witness data.
    #[arg(long = "witness")]
    witness: Vec<String>,
}

fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    if !hex.len().is_multiple_of(2) {
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

fn build_transaction(cli: &Cli) -> Result<Transaction, Box<dyn Error>> {
    let mut inputs: Vec<TxInput> = cli
        .inputs
        .iter()
        .map(|raw| parse_input(raw))
        .collect::<Result<_, _>>()?;

    let outputs: Vec<TxOutput> = cli
        .outputs
        .iter()
        .map(|raw| parse_output(raw))
        .collect::<Result<_, _>>()?;

    for raw in &cli.witness {
        let (index, items) = parse_witness(raw)?;
        let input_count = inputs.len();
        let input = inputs.get_mut(index).ok_or_else(|| {
        err(format!(
            "--witness '{raw}': input index {index} out of range (only {input_count} input(s) given)"
        ))
    })?;
        input.witness = items;
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
    let args: Vec<String> = std::env::args().collect();

    let trx = if args.len() == 1 {
        // no flags given — walk the user through it
        run_interactive()?
    } else {
        let cli = Cli::parse();
        build_transaction(&cli)?
    };

    let serialized = serialize_transaction(&trx);
    println!("Serialized transaction:");
    println!("{:?}", serialized);
    println!("Serialized Hex transaction:");
    println!("{}", bytes_to_hex(&serialized));
    println!("\nTransaction size: {} bytes", serialized.len());
    Ok(())
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn parse_input(raw: &str) -> Result<TxInput, Box<dyn Error>> {
    let parts: Vec<&str> = raw.split(':').collect();
    if parts.len() != 4 {
        return Err(err(format!(
            "--input '{raw}' must have 4 colon-separated fields: prev_txid:vout:sequence:script_sig"
        )));
    }

    let prev_txid = hex_to_bytes(parts[0])
        .map_err(|e| err(format!("--input '{raw}': invalid prev_txid hex: {e}")))?;
    let vout: u32 = parts[1]
        .parse()
        .map_err(|_| err(format!("--input '{raw}': vout must be a valid u32")))?;
    let sequence: u32 = parts[2]
        .parse()
        .map_err(|_| err(format!("--input '{raw}': sequence must be a valid u32")))?;
    let script_sig = hex_to_bytes(parts[3])
        .map_err(|e| err(format!("--input '{raw}': invalid script_sig hex: {e}")))?;

    Ok(TxInput {
        prev_txid,
        vout,
        script_sig,
        sequence,
        witness: Vec::new(),
    })
}

fn parse_output(raw: &str) -> Result<TxOutput, Box<dyn Error>> {
    let parts: Vec<&str> = raw.split(':').collect();
    if parts.len() != 2 {
        return Err(err(format!(
            "--output '{raw}' must have 2 colon-separated fields: value:script_pubkey"
        )));
    }

    let value: u64 = parts[0]
        .parse()
        .map_err(|_| err(format!("--output '{raw}': value must be a valid u64")))?;
    let script_pubkey = hex_to_bytes(parts[1])
        .map_err(|e| err(format!("--output '{raw}': invalid script_pubkey hex: {e}")))?;

    Ok(TxOutput {
        value,
        script_pubkey,
    })
}

/// Returns (input_index, witness_items).
type WitnessEntry = (usize, Vec<Vec<u8>>);
fn parse_witness(raw: &str) -> Result<WitnessEntry, Box<dyn Error>> {
    let (index_part, items_part) = raw
        .split_once(':')
        .ok_or_else(|| err(format!("--witness '{raw}' must be '<input_index>:<items>'")))?;

    let index: usize = index_part.parse().map_err(|_| {
        err(format!(
            "--witness '{raw}': input_index must be a valid number"
        ))
    })?;

    let items = if items_part.is_empty() {
        Vec::new()
    } else {
        items_part
            .split(',')
            .map(|item| {
                hex_to_bytes(item)
                    .map_err(|e| err(format!("--witness '{raw}': invalid item hex: {e}")))
            })
            .collect::<Result<Vec<_>, _>>()?
    };

    Ok((index, items))
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

fn prompt(label: &str) -> Result<String, Box<dyn Error>> {
    print!("{label}: ");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

fn run_interactive() -> Result<Transaction, Box<dyn Error>> {
    let version: i32 = prompt("Transaction version")?.parse()?;
    let locktime: u32 = prompt("Locktime")?.parse()?;
    let segwit = prompt("Is this a SegWit transaction? (y/n)")?.eq_ignore_ascii_case("y");

    let mut inputs = Vec::new();
    loop {
        let raw = prompt("Input as prev_txid:vout:sequence:script_sig (blank to finish)")?;
        if raw.is_empty() {
            break;
        }
        match parse_input(&raw) {
            Ok(input) => inputs.push(input),
            Err(e) => println!("  invalid input, try again: {e}"),
        }
    }

    let mut outputs = Vec::new();
    loop {
        let raw = prompt("Output as value:script_pubkey (blank to finish)")?;
        if raw.is_empty() {
            break;
        }
        match parse_output(&raw) {
            Ok(output) => outputs.push(output),
            Err(e) => println!("  invalid output, try again: {e}"),
        }
    }

    if segwit {
        loop {
            let raw = prompt("Witness as input_index:item1,item2,... (blank to finish)")?;
            if raw.is_empty() {
                break;
            }
            match parse_witness(&raw) {
                Ok((index, items)) => {
                    if let Some(input) = inputs.get_mut(index) {
                        input.witness = items;
                    } else {
                        println!("  input index {index} out of range, ignoring");
                    }
                }
                Err(e) => println!("  invalid witness data, try again: {e}"),
            }
        }
    }

    Ok(Transaction {
        version,
        inputs,
        outputs,
        locktime,
        segwit,
    })
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
