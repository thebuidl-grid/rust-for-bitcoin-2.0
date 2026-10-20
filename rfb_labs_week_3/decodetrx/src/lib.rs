use sha2::{Digest, Sha256};
use std::fmt;

use transaction::{Amount, Input, Output, Transaction, Txid};

mod transaction;

#[derive(Debug)]
pub struct BuildError(String);

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for BuildError {}

pub fn parse_hex(value: &str, field: &str) -> Result<Vec<u8>, BuildError> {
    let value = value.trim();

    if value.is_empty() {
        return Err(BuildError(format!("{} cannot be empty", field)));
    }

    if !value.len().is_multiple_of(2) {
        return Err(BuildError(format!(
            "{} must contain an even number of hexadecimal characters",
            field
        )));
    }

    if !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(BuildError(format!(
            "{} contains invalid hexadecimal characters: '{}'",
            field, value
        )));
    }

    hex::decode(value).map_err(|e| BuildError(format!("invalid hexadecimal {}: {}", field, e)))
}

pub fn parse_txid(value: &str) -> Result<Txid, BuildError> {
    let bytes = parse_hex(value, "TXID")?;

    if bytes.len() != 32 {
        return Err(BuildError(format!(
            "TXID must be exactly 32 bytes (64 hex characters), got {} bytes",
            bytes.len()
        )));
    }

    let mut txid = [0u8; 32];

    for (i, byte) in bytes.iter().enumerate() {
        txid[31 - i] = *byte;
    }

    Ok(Txid::from_bytes(txid))
}

pub fn parse_input(value: &str) -> Result<Input, BuildError> {
    let parts: Vec<&str> = value.splitn(4, ':').collect();

    if parts.len() != 4 {
        return Err(BuildError(
            "input must use TXID:VOUT:SEQUENCE:SCRIPTSIG".to_string(),
        ));
    }

    let txid = parse_txid(parts[0])?;

    let output_index = parts[1]
        .parse::<u32>()
        .map_err(|_| BuildError(format!("invalid input output index '{}'", parts[1])))?;

    let sequence = parse_u32(parts[2], "input sequence")?;

    let script_sig = if parts[3].is_empty() {
        Vec::new()
    } else {
        parse_hex(parts[3], "scriptSig")?
    };

    Ok(Input {
        txid,
        output_index,
        script_sig,
        sequence,
        witness: Vec::new(),
    })
}

pub fn parse_output(value: &str) -> Result<Output, BuildError> {
    let parts: Vec<&str> = value.splitn(2, ':').collect();

    if parts.len() != 2 {
        return Err(BuildError(
            "output must use AMOUNT_IN_SATOSHIS:SCRIPTPUBKEY".to_string(),
        ));
    }

    let amount = parts[0].parse::<u64>().map_err(|_| {
        BuildError(format!(
            "invalid output amount '{}'; amount must be satoshis",
            parts[0]
        ))
    })?;

    let script_pubkey = parse_hex(parts[1], "scriptPubKey")?;

    Ok(Output {
        amount: Amount::from_sat(amount),
        script_pubkey,
    })
}

fn parse_u32(value: &str, field: &str) -> Result<u32, BuildError> {
    if let Some(hex_value) = value.strip_prefix("0x") {
        u32::from_str_radix(hex_value, 16)
            .map_err(|_| BuildError(format!("invalid hexadecimal {} '{}'", field, value)))
    } else {
        value
            .parse::<u32>()
            .map_err(|_| BuildError(format!("invalid {} '{}'", field, value)))
    }
}

pub fn add_witness(inputs: &mut [Input], value: &str) -> Result<(), BuildError> {
    let parts: Vec<&str> = value.splitn(2, ':').collect();

    if parts.len() != 2 {
        return Err(BuildError(
            "witness must use INPUT_INDEX:ITEM_HEX".to_string(),
        ));
    }

    let input_index = parts[0]
        .parse::<usize>()
        .map_err(|_| BuildError(format!("invalid witness input index '{}'", parts[0])))?;

    if input_index >= inputs.len() {
        return Err(BuildError(format!(
            "witness input index {} is out of range; there are {} inputs",
            input_index,
            inputs.len()
        )));
    }

    let item = parse_hex(parts[1], "witness item")?;

    inputs[input_index].witness.push(item);

    Ok(())
}

fn encode_compact_size(value: u64) -> Vec<u8> {
    match value {
        0..=252 => vec![value as u8],

        253..=65535 => {
            let mut result = vec![253];
            result.extend_from_slice(&(value as u16).to_le_bytes());
            result
        }

        65536..=4294967295 => {
            let mut result = vec![254];
            result.extend_from_slice(&(value as u32).to_le_bytes());
            result
        }

        _ => {
            let mut result = vec![255];
            result.extend_from_slice(&value.to_le_bytes());
            result
        }
    }
}

fn serialize_input(input: &Input) -> Vec<u8> {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&input.txid.0);
    bytes.extend_from_slice(&input.output_index.to_le_bytes());
    bytes.extend_from_slice(&encode_compact_size(input.script_sig.len() as u64));
    bytes.extend_from_slice(&input.script_sig);
    bytes.extend_from_slice(&input.sequence.to_le_bytes());

    bytes
}

fn serialize_output(output: &Output) -> Vec<u8> {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&output.amount.0.to_le_bytes());
    bytes.extend_from_slice(&encode_compact_size(output.script_pubkey.len() as u64));
    bytes.extend_from_slice(&output.script_pubkey);

    bytes
}

fn serialize_for_txid(
    version: u32,
    inputs: &[Input],
    outputs: &[Output],
    lock_time: u32,
) -> Vec<u8> {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(&version.to_le_bytes());
    bytes.extend_from_slice(&encode_compact_size(inputs.len() as u64));

    for input in inputs {
        bytes.extend_from_slice(&serialize_input(input));
    }

    bytes.extend_from_slice(&encode_compact_size(outputs.len() as u64));

    for output in outputs {
        bytes.extend_from_slice(&serialize_output(output));
    }

    bytes.extend_from_slice(&lock_time.to_le_bytes());

    bytes
}

pub fn serialize_transaction(
    version: u32,
    inputs: &[Input],
    outputs: &[Output],
    lock_time: u32,
) -> Vec<u8> {
    let has_witness = inputs.iter().any(|input| !input.witness.is_empty());

    let mut bytes = Vec::new();

    bytes.extend_from_slice(&version.to_le_bytes());

    if has_witness {
        bytes.push(0x00);
        bytes.push(0x01);
    }

    bytes.extend_from_slice(&encode_compact_size(inputs.len() as u64));

    for input in inputs {
        bytes.extend_from_slice(&serialize_input(input));
    }

    bytes.extend_from_slice(&encode_compact_size(outputs.len() as u64));

    for output in outputs {
        bytes.extend_from_slice(&serialize_output(output));
    }

    if has_witness {
        for input in inputs {
            bytes.extend_from_slice(&encode_compact_size(input.witness.len() as u64));

            for item in &input.witness {
                bytes.extend_from_slice(&encode_compact_size(item.len() as u64));
                bytes.extend_from_slice(item);
            }
        }
    }

    bytes.extend_from_slice(&lock_time.to_le_bytes());

    bytes
}

fn hash_transaction(bytes: &[u8]) -> Txid {
    let first_hash = Sha256::digest(bytes);
    let second_hash = Sha256::digest(first_hash);

    let result: [u8; 32] = second_hash.into();

    Txid::from_bytes(result)
}

pub fn build_transaction(
    version: u32,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    lock_time: u32,
) -> Result<(String, usize, Transaction), BuildError> {
    if inputs.is_empty() {
        return Err(BuildError(
            "transaction must contain at least one input".to_string(),
        ));
    }

    if outputs.is_empty() {
        return Err(BuildError(
            "transaction must contain at least one output".to_string(),
        ));
    }

    let serialized = serialize_transaction(version, &inputs, &outputs, lock_time);

    let txid_bytes = serialize_for_txid(version, &inputs, &outputs, lock_time);

    let transaction_id = hash_transaction(&txid_bytes);

    let transaction = Transaction {
        transaction_id,
        version,
        inputs,
        outputs,
        lock_time,
    };

    Ok((hex::encode(&serialized), serialized.len(), transaction))
}
