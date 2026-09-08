use crate::{encoding::encode_varint, error::SerializeError};

#[derive(Debug)]
pub struct TxInput {
    pub prev_txid: Vec<u8>,
    pub vout: u32,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
    pub witness: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub struct TxOutput {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
}

#[derive(Debug)]
pub struct Transaction {
    pub version: i32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub locktime: u32,
}

impl Transaction {
    pub fn is_segwit(&self) -> bool {
        self.inputs.iter().any(|input| !input.witness.is_empty())
    }
}

pub fn serialize_transaction(trx: &Transaction) -> Result<Vec<u8>, SerializeError> {
    if trx.inputs.is_empty() {
        return Err(SerializeError::NoInputs);
    }

    if trx.outputs.is_empty() {
        return Err(SerializeError::NoOutputs);
    }

    for (input_index, input) in trx.inputs.iter().enumerate() {
        if input.prev_txid.len() != 32 {
            return Err(SerializeError::InvalidTxidLength {
                input_index,
                actual_length: input.prev_txid.len(),
            });
        }
    }

    let mut result = Vec::new();

    result.extend_from_slice(&trx.version.to_le_bytes());

    if trx.is_segwit() {
        result.push(0x00);
        result.push(0x01);
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

    if trx.is_segwit() {
        for input in &trx.inputs {
            result.extend_from_slice(&encode_varint(input.witness.len()));

            for item in &input.witness {
                result.extend_from_slice(&encode_varint(item.len()));
                result.extend_from_slice(item);
            }
        }
    }

    result.extend_from_slice(&trx.locktime.to_le_bytes());

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(previous_txid: Vec<u8>, witness: Vec<Vec<u8>>) -> TxInput {
        TxInput {
            prev_txid: previous_txid,
            vout: 1,
            script_sig: Vec::new(),
            sequence: 0xffff_fffd,
            witness,
        }
    }

    fn output() -> TxOutput {
        TxOutput {
            value: 500,
            script_pubkey: vec![0x51],
        }
    }

    #[test]
    fn rejects_a_transaction_without_inputs() {
        let transaction = Transaction {
            version: 1,
            inputs: Vec::new(),
            outputs: vec![output()],
            locktime: 0,
        };

        let error = serialize_transaction(&transaction).unwrap_err();

        assert!(matches!(error, SerializeError::NoInputs));
    }

    #[test]
    fn rejects_a_transaction_without_outputs() {
        let transaction = Transaction {
            version: 1,
            inputs: vec![input(vec![0x00; 32], Vec::new())],
            outputs: Vec::new(),
            locktime: 0,
        };

        let error = serialize_transaction(&transaction).unwrap_err();

        assert!(matches!(error, SerializeError::NoOutputs));
    }

    #[test]
    fn rejects_an_input_with_an_invalid_txid_length() {
        let transaction = Transaction {
            version: 1,
            inputs: vec![input(vec![0x00; 31], Vec::new())],
            outputs: vec![output()],
            locktime: 0,
        };

        let error = serialize_transaction(&transaction).unwrap_err();

        assert!(matches!(
            error,
            SerializeError::InvalidTxidLength {
                input_index: 0,
                actual_length: 31,
            }
        ));
    }

    #[test]
    fn serializes_a_legacy_transaction_without_segwit_fields() {
        let transaction = Transaction {
            version: 1,
            inputs: vec![input(vec![0x00; 32], Vec::new())],
            outputs: vec![output()],
            locktime: 0,
        };

        let serialized = serialize_transaction(&transaction).unwrap();

        assert_eq!(
            hex::encode(serialized),
            "010000000100000000000000000000000000000000000000000000000000000000000000000100000000fdffffff01f401000000000000015100000000"
        );
    }

    #[test]
    fn serializes_a_segwit_transaction_with_its_witness() {
        let transaction = Transaction {
            version: 2,
            inputs: vec![input(vec![0x11; 32], vec![vec![0xaa]])],
            outputs: vec![output()],
            locktime: 0,
        };

        let serialized = serialize_transaction(&transaction).unwrap();

        assert_eq!(
            hex::encode(serialized),
            "0200000000010111111111111111111111111111111111111111111111111111111111111111110100000000fdffffff01f40100000000000001510101aa00000000"
        );
    }
}
