use clap::Parser;
use std::error::Error;

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


/// Construct and serialize a Bitcoin transaction from command-line arguments.
#[derive(Parser, Debug)]
#[command(name = "serializeTrx", about = "Construct and serialize a Bitcoin transaction")]
struct Cli {
    /// Transaction version
    #[arg(long, default_value_t = 2)]
    version: i32,

    /// Locktime
    #[arg(long, default_value_t = 0)]
    locktime: u32,

    /// Mark the transaction as SegWit (adds marker/flag bytes and witness data)
    #[arg(long)]
    segwit: bool,

    /// Transaction input, repeatable: TXID_HEX:VOUT:SEQUENCE[:SCRIPTSIG_HEX]
    #[arg(long = "input", value_name = "TXID:VOUT:SEQUENCE[:SCRIPTSIG_HEX]")]
    inputs: Vec<String>,

    /// Transaction output, repeatable: VALUE_SATS:SCRIPTPUBKEY_HEX
    #[arg(long = "output", value_name = "VALUE:SCRIPTPUBKEY_HEX")]
    outputs: Vec<String>,

    /// Witness item, repeatable and order-sensitive per input: INPUT_INDEX:ITEM_HEX
    #[arg(long = "witness", value_name = "INPUT_INDEX:ITEM_HEX")]
    witnesses: Vec<String>,
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

/// Validates that a string is well-formed hexadecimal, then converts it to bytes.
/// `field_name` is used to produce a meaningful, field-specific error message.
fn parse_hex_field(field_name: &str, hex: &str) -> Result<Vec<u8>, String> {
    if hex.is_empty() {
        return Ok(Vec::new());
    }
    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!(
            "{field_name}: '{hex}' contains non-hexadecimal characters"
        ));
    }
    hex_to_bytes(hex).map_err(|e| format!("{field_name}: {e}"))
}

/// Parses "TXID_HEX:VOUT:SEQUENCE[:SCRIPTSIG_HEX]" into a TxInput.
fn parse_input(spec: &str) -> Result<TxInput, String> {
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.len() < 3 || parts.len() > 4 {
        return Err(format!(
            "invalid --input '{spec}': expected TXID:VOUT:SEQUENCE[:SCRIPTSIG_HEX]"
        ));
    }

    let prev_txid = parse_hex_field("input txid", parts[0])?;
    if prev_txid.len() != 32 {
        return Err(format!(
            "invalid --input '{spec}': txid must be 32 bytes (64 hex chars), got {} bytes",
            prev_txid.len()
        ));
    }

    let vout: u32 = parts[1].parse().map_err(|_| {
        format!("invalid --input '{spec}': vout '{}' is not a valid u32", parts[1])
    })?;

    let sequence: u32 = parts[2].parse().map_err(|_| {
        format!(
            "invalid --input '{spec}': sequence '{}' is not a valid u32",
            parts[2]
        )
    })?;

    let script_sig = if parts.len() == 4 {
        parse_hex_field("scriptSig", parts[3])?
    } else {
        Vec::new()
    };

    Ok(TxInput {
        prev_txid,
        vout,
        script_sig,
        sequence,
        witness: Vec::new(),
    })
}

/// Parses "VALUE_SATS:SCRIPTPUBKEY_HEX" into a TxOutput.
fn parse_output(spec: &str) -> Result<TxOutput, String> {
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.len() != 2 {
        return Err(format!(
            "invalid --output '{spec}': expected VALUE:SCRIPTPUBKEY_HEX"
        ));
    }

    let value: u64 = parts[0].parse().map_err(|_| {
        format!(
            "invalid --output '{spec}': value '{}' is not a valid u64",
            parts[0]
        )
    })?;

    let script_pubkey = parse_hex_field("scriptPubKey", parts[1])?;

    Ok(TxOutput { value, script_pubkey })
}

/// Parses "INPUT_INDEX:ITEM_HEX" into (input index, witness item bytes).
fn parse_witness(spec: &str) -> Result<(usize, Vec<u8>), String> {
    let parts: Vec<&str> = spec.split(':').collect();
    if parts.len() != 2 {
        return Err(format!(
            "invalid --witness '{spec}': expected INPUT_INDEX:ITEM_HEX"
        ));
    }

    let index: usize = parts[0].parse().map_err(|_| {
        format!(
            "invalid --witness '{spec}': input index '{}' is not a valid number",
            parts[0]
        )
    })?;

    let item = parse_hex_field("witness item", parts[1])?;

    Ok((index, item))
}


fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    if cli.inputs.is_empty() {
        return Err("at least one --input is required".into());
    }
    if cli.outputs.is_empty() {
        return Err("at least one --output is required".into());
    }
    if !cli.segwit && !cli.witnesses.is_empty() {
        return Err("--witness was provided but --segwit was not set".into());
    }

    let mut inputs = Vec::new();
    for spec in &cli.inputs {
        inputs.push(parse_input(spec)?);
    }

    let outputs = cli
        .outputs
        .iter()
        .map(|spec| parse_output(spec))
        .collect::<Result<Vec<_>, _>>()?;

    for spec in &cli.witnesses {
        let (index, item) = parse_witness(spec)?;
        let input_count = inputs.len();
        let input = inputs.get_mut(index).ok_or_else(|| {
            format!(
                "invalid --witness '{spec}': input index {index} does not exist ({input_count} input(s) defined)"
            )
        })?;
        input.witness.push(item);
    }

    let trx = Transaction {
        version: cli.version,
        inputs,
        outputs,
        locktime: cli.locktime,
        segwit: cli.segwit,
    };

    // Serialize
    let serialized = serialize_transaction(&trx);

    println!("Serialized transaction:");
    println!("{:?}", serialized);
    println!("Serialized Hex transaction:");
    println!("{}", bytes_to_hex(&serialized));

    println!("\nTransaction size: {} bytes", serialized.len());

    Ok(())
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
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
