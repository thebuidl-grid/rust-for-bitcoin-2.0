/// A single transaction input, including its optional witness stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxInput {
    pub prev_txid: Vec<u8>,
    pub vout: u32,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
    pub witness: Vec<Vec<u8>>,
}

/// A single transaction output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxOutput {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
}

/// A full transaction, ready for serialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub version: i32,
    pub segwit: bool,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub locktime: u32,
}

impl Transaction {
    /// Serialize the transaction to raw bytes following the legacy/SegWit
    /// wire format (BIP-144 marker+flag and witness stacks when `segwit` is set).
    pub fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::new();

        result.extend_from_slice(&self.version.to_le_bytes());

        if self.segwit {
            result.push(0x00);
            result.push(0x01);
        }

        result.extend_from_slice(&encode_varint(self.inputs.len()));
        for input in &self.inputs {
            result.extend_from_slice(&input.prev_txid);
            result.extend_from_slice(&input.vout.to_le_bytes());
            result.extend_from_slice(&encode_varint(input.script_sig.len()));
            result.extend_from_slice(&input.script_sig);
            result.extend_from_slice(&input.sequence.to_le_bytes());
        }

        result.extend_from_slice(&encode_varint(self.outputs.len()));
        for output in &self.outputs {
            result.extend_from_slice(&output.value.to_le_bytes());
            result.extend_from_slice(&encode_varint(output.script_pubkey.len()));
            result.extend_from_slice(&output.script_pubkey);
        }

        if self.segwit {
            for input in &self.inputs {
                result.extend_from_slice(&encode_varint(input.witness.len()));
                for item in &input.witness {
                    result.extend_from_slice(&encode_varint(item.len()));
                    result.extend_from_slice(item);
                }
            }
        }

        result.extend_from_slice(&self.locktime.to_le_bytes());

        result
    }
}

/// Encode a length as a Bitcoin CompactSize ("varint").
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

/// Render bytes as a lowercase hex string.
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
