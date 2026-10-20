use clap::Parser;
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

#[derive(Parser)]
#[command(
    name = "serializeTrx",
    about = "Construct and serialize a Bitcoin transaction from CLI arguments"
)]
struct Cli {
    #[arg(long, default_value_t = 2)]
    version: i32,

    #[arg(long)]
    segwit: bool,

    #[arg(long = "input")]
    inputs: Vec<String>,

    #[arg(long = "output")]
    outputs: Vec<String>,

    #[arg(long = "witness")]
    witness: Vec<String>,

    #[arg(long, default_value_t = 0)]
    locktime: u32,
}

#[derive(Debug)]
enum CliError {
    InvalidHex { field: String, value: String },
    InvalidNumber { field: String, value: String },
    InvalidTxid { value: String, byte_len: usize },
    MalformedInput { value: String },
    MalformedOutput { value: String },
    MalformedWitness { value: String },
    WitnessIndexOutOfRange { index: usize, input_count: usize },
    NoInputs,
    NoOutputs,
    WitnessWithoutSegwit,
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

// fn main() -> Result<(), Box<dyn Error>> {

//     let input = TxInput {
//         prev_txid: hex_to_bytes(
//             "8fb0d07bb3766421bff2d908b70e5de818e4d85a436ea3606310c1052b0dc821"
//         )?,
//         vout: 1,
//         script_sig: vec![],
//         sequence: 0xffffffff,
//         witness: vec![
//             hex_to_bytes("3045022100f8704a3e7d55d4b5ee448cc6365caeffa42c2b00f74a37726d4fa3c11982e3e502203591c4a4bde9200281755ae5a8759116ce6e0cc7f5d30cf0eeb5b2b74f74bab301")?,
//             hex_to_bytes("029cbb1e568de08f469a8751aa2000331f130ca92ad49012d9cececaf6f8eb2358")?
//         ]
//     };

//     let output_0 = TxOutput {
//         value: 69886,
//         script_pubkey: hex_to_bytes("0014a632c1fff47af29f8c81dc4c6e91eb49a116c12b")?,
//     };

//     let output_1 = TxOutput {
//         value: 29442,
//         script_pubkey: hex_to_bytes("00149831122b93d21715c70db626ccc844d3c21f9687")?,
//     };

//     let trx = Transaction {
//         version : 2,
//         inputs: vec![input],
//         outputs: vec![output_0, output_1],
//         locktime: 0,
//         segwit: true
//     };

//        // Serialize
//     let serialized = serialize_transaction(&trx);

//     println!("Serialized transaction:");
//     println!("{:?}", &serialized);
//     println!("Serialized Hex transaction:");
//     println!("{}", bytes_to_hex(&serialized));

//     println!("\nTransaction size: {} bytes", serialized.len());

//     Ok(())

// }

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::InvalidHex { field, value } => {
                write!(f, "'{field}' is not valid hex: \"{value}\"")
            }
            CliError::InvalidNumber { field, value } => {
                write!(f, "'{field}' is not a valid number: \"{value}\"")
            }
            CliError::InvalidTxid { value, byte_len } => write!(
                f,
                "txid \"{value}\" decodes to {byte_len} bytes, but a txid must be exactly 32 bytes"
            ),
            CliError::MalformedInput { value } => write!(
                f,
                "--input \"{value}\" must have the form txid:vout:sequence:script_sig_hex"
            ),
            CliError::MalformedOutput { value } => write!(
                f,
                "--output \"{value}\" must have the form value_sats:script_pubkey_hex"
            ),
            CliError::MalformedWitness { value } => write!(
                f,
                "--witness \"{value}\" must have the form input_index:item_hex"
            ),
            CliError::WitnessIndexOutOfRange { index, input_count } => write!(
                f,
                "--witness targets input index {index}, but only {input_count} input(s) were provided"
            ),
            CliError::NoInputs => write!(f, "at least one --input is required"),
            CliError::NoOutputs => write!(f, "at least one --output is required"),
            CliError::WitnessWithoutSegwit => write!(
                f,
                "--witness was provided but --segwit was not set; SegWit inputs are the only ones with witness data"
            ),
        }
    }
}

impl std::error::Error for CliError {}

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

fn parse_input(raw: &str) -> Result<TxInput, CliError> {
    let parts: Vec<&str> = raw.splitn(4, ':').collect();
    let [txid_hex, vout_str, sequence_str, script_sig_hex] = parts[..] else {
        return Err(CliError::MalformedInput {
            value: raw.to_string(),
        });
    };

    let prev_txid = hex_to_bytes(txid_hex).map_err(|_| CliError::InvalidHex {
        field: "input.txid".to_string(),
        value: txid_hex.to_string(),
    })?;
    if prev_txid.len() != 32 {
        return Err(CliError::InvalidTxid {
            value: txid_hex.to_string(),
            byte_len: prev_txid.len(),
        });
    }

    let vout = vout_str
        .parse::<u32>()
        .map_err(|_| CliError::InvalidNumber {
            field: "input.vout".to_string(),
            value: vout_str.to_string(),
        })?;
    let sequence = sequence_str
        .parse::<u32>()
        .map_err(|_| CliError::InvalidNumber {
            field: "input.sequence".to_string(),
            value: sequence_str.to_string(),
        })?;
    let script_sig = hex_to_bytes(script_sig_hex).map_err(|_| CliError::InvalidHex {
        field: "input.script_sig".to_string(),
        value: script_sig_hex.to_string(),
    })?;

    Ok(TxInput {
        prev_txid,
        vout,
        script_sig,
        sequence,
        witness: vec![],
    })
}

fn parse_output(raw: &str) -> Result<TxOutput, CliError> {
    let parts: Vec<&str> = raw.splitn(2, ':').collect();
    let [value_str, script_pubkey_hex] = parts[..] else {
        return Err(CliError::MalformedOutput {
            value: raw.to_string(),
        });
    };

    let value = value_str
        .parse::<u64>()
        .map_err(|_| CliError::InvalidNumber {
            field: "output.value".to_string(),
            value: value_str.to_string(),
        })?;
    let script_pubkey = hex_to_bytes(script_pubkey_hex).map_err(|_| CliError::InvalidHex {
        field: "output.script_pubkey".to_string(),
        value: script_pubkey_hex.to_string(),
    })?;

    Ok(TxOutput {
        value,
        script_pubkey,
    })
}

fn parse_witness(raw: &str) -> Result<(usize, Vec<u8>), CliError> {
    let parts: Vec<&str> = raw.splitn(2, ':').collect();
    let [index_str, item_hex] = parts[..] else {
        return Err(CliError::MalformedWitness {
            value: raw.to_string(),
        });
    };

    let index = index_str
        .parse::<usize>()
        .map_err(|_| CliError::InvalidNumber {
            field: "witness.input_index".to_string(),
            value: index_str.to_string(),
        })?;
    let item = hex_to_bytes(item_hex).map_err(|_| CliError::InvalidHex {
        field: "witness.item".to_string(),
        value: item_hex.to_string(),
    })?;

    Ok((index, item))
}

fn run(cli: Cli) -> Result<(), CliError> {
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

    if inputs.is_empty() {
        return Err(CliError::NoInputs);
    }
    if outputs.is_empty() {
        return Err(CliError::NoOutputs);
    }

    // for raw in &cli.witness {
    //     let (index, item) = parse_witness(raw)?;
    //     let input = inputs
    //         .get_mut(index)
    //         .ok_or(CliError::WitnessIndexOutOfRange { index, input_count: inputs.len() })?;
    //     input.witness.push(item);
    // }

    for raw in &cli.witness {
        let (index, item) = parse_witness(raw)?;
        let input_count = inputs.len();
        let input = inputs
            .get_mut(index)
            .ok_or(CliError::WitnessIndexOutOfRange { index, input_count })?;
        input.witness.push(item);
    }

    if !cli.segwit && inputs.iter().any(|input| !input.witness.is_empty()) {
        return Err(CliError::WitnessWithoutSegwit);
    }

    let trx = Transaction {
        version: cli.version,
        inputs,
        outputs,
        locktime: cli.locktime,
        segwit: cli.segwit,
    };

    let serialized = serialize_transaction(&trx);
    println!(
        "Serialized transaction (hex): {}",
        bytes_to_hex(&serialized)
    );
    println!("Transaction size: {} bytes", serialized.len());

    Ok(())
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        eprintln!("Error: {err}");
        std::process::exit(1);
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
