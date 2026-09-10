use crate::transaction::Transaction;

/// Converts hex string to bytes with validation
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, crate::error::TxSerializerError> {
    if hex.len() % 2 != 0 {
        return Err(crate::error::TxSerializerError::OddHexLength(hex.len()));
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);

    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16)?;
        bytes.push(byte);
    }

    Ok(bytes)
}

/// Converts bytes to hex string
// pub fn bytes_to_hex(bytes: &[u8]) -> String {
//     bytes.iter().map(|b| format!("{:02x}", b)).collect()
// }
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut result = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(result, "{:02x}", b);
    }
    result
}

/// Serializes a Bitcoin transaction according to BIP144 (SegWit)
pub fn serialize_transaction(trx: &Transaction) -> Vec<u8> {
    let mut result = Vec::new();

    // Version (4 bytes, little-endian)
    result.extend_from_slice(&trx.version.to_le_bytes());

    // SegWit marker and flag
    if trx.segwit {
        result.push(0x00); // marker
        result.push(0x01); // flag
    }

    // Input count (VarInt)
    result.extend_from_slice(&encode_varint(trx.inputs.len()));

    // Inputs
    for input in &trx.inputs {
        // Previous transaction ID
        result.extend_from_slice(&input.prev_txid);

        // Previous output index (4 bytes, little-endian)
        result.extend_from_slice(&input.vout.to_le_bytes());

        // ScriptSig length (VarInt)
        result.extend_from_slice(&encode_varint(input.script_sig.len()));

        // ScriptSig
        result.extend_from_slice(&input.script_sig);

        // Sequence (4 bytes, little-endian)
        result.extend_from_slice(&input.sequence.to_le_bytes());
    }

    // Output count (VarInt)
    result.extend_from_slice(&encode_varint(trx.outputs.len()));

    // Outputs
    for output in &trx.outputs {
        // Value in satoshis (8 bytes, little-endian)
        result.extend_from_slice(&output.value.to_le_bytes());

        // ScriptPubKey length (VarInt)
        result.extend_from_slice(&encode_varint(output.script_pubkey.len()));

        // ScriptPubKey
        result.extend_from_slice(&output.script_pubkey);
    }

    // Witness data
    if trx.segwit {
        for input in &trx.inputs {
            // Number of witness items (VarInt)
            result.extend_from_slice(&encode_varint(input.witness.len()));

            for item in &input.witness {
                // Witness item length (VarInt)
                result.extend_from_slice(&encode_varint(item.len()));

                // Witness item
                result.extend_from_slice(item);
            }
        }
    }

    // Locktime (4 bytes, little-endian)
    result.extend_from_slice(&trx.locktime.to_le_bytes());

    result
}

/// Encodes a value as Bitcoin VarInt (CompactSize)
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