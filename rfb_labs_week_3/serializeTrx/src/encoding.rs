use crate::error::SerializeError;

/// Decode human-readable hexadecimal text into bytes.
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, SerializeError> {
    Ok(hex::decode(hex)?)
}

/// Encode bytes as lowercase hexadecimal text.
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Encode a count or byte length using Bitcoin's CompactSize format.
///
/// CompactSize is used for input and output counts, script lengths, witness
/// item counts, and witness item lengths.
pub fn encode_varint(value: usize) -> Vec<u8> {
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
