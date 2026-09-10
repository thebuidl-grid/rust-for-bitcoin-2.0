use crate::error::TxSerializerError;

#[derive(Debug, Clone)]
pub struct TxInput {
    pub prev_txid: Vec<u8>,
    pub vout: u32,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
    pub witness: Vec<Vec<u8>>,
}

impl TxInput {
    pub fn new(
        prev_txid: Vec<u8>,
        vout: u32,
        script_sig: Vec<u8>,
        sequence: u32,
    ) -> Result<Self, TxSerializerError> {
        if prev_txid.len() != 32 {
            return Err(TxSerializerError::InvalidTxIdLength(prev_txid.len()));
        }

        Ok(TxInput {
            prev_txid,
            vout,
            script_sig,
            sequence,
            witness: Vec::new(),
        })
    }

    pub fn add_witness_item(&mut self, item: Vec<u8>) {
        self.witness.push(item);
    }

    pub fn get_txid_hex(&self) -> String {
        self.prev_txid
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct TxOutput {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
}

impl TxOutput {
    pub fn new(value: u64, script_pubkey: Vec<u8>) -> Self {
        TxOutput {
            value,
            script_pubkey,
        }
    }

    pub fn get_script_hex(&self) -> String {
        self.script_pubkey
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[derive(Debug)]
pub struct Transaction {
    pub version: i32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub locktime: u32,
    pub segwit: bool,
}

impl Transaction {
    pub fn new(
        version: i32,
        inputs: Vec<TxInput>,
        outputs: Vec<TxOutput>,
        locktime: u32,
        segwit: bool,
    ) -> Result<Self, TxSerializerError> {
        if inputs.is_empty() {
            return Err(TxSerializerError::NoInputs);
        }

        if outputs.is_empty() {
            return Err(TxSerializerError::NoOutputs);
        }

        if segwit {
            if !inputs.iter().any(|i| !i.witness.is_empty()) {
                return Err(TxSerializerError::ArgError(
                    "SegWit enabled but no input has witness data".to_string(),
                ));
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
}
