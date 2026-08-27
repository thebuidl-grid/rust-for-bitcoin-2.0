use std::fmt;

use crate::error::TransactionError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    P2pkh,
    P2wpkh,
    P2tr,
    OpReturn,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TxOutput {
    pub value: u64,
    pub recipient: String,
    pub output_type: OutputType,
}

#[derive(Debug, PartialEq, Eq)]
pub struct OutPoint {
    pub txid: String,
    pub vout: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InputKind {
    Regular {
        previous_output: OutPoint,
        value: u64,
        sequence: u32,
    },
    Coinbase {
        block_height: u32,
        reward: u64,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Transaction {
    pub version: i32,
    pub inputs: Vec<InputKind>,
    pub outputs: Vec<TxOutput>,
    pub locktime: u32,
}

pub trait BitcoinValue {
    fn value(&self) -> u64;

    fn value_in_btc(&self) -> f64 {
        self.value() as f64 / 100_000_000.0
    }
}

impl Transaction {
    pub fn new(version: i32, locktime: u32) -> Self {
        Self {
            version,
            inputs: Vec::new(),
            outputs: Vec::new(),
            locktime,
        }
    }

    pub fn add_input(&mut self, input: InputKind) {
        // TODO(Part 3): move `input` into the transaction.
        // let _ = input;

        self.inputs.push(input);
    }

    pub fn add_output(&mut self, output: TxOutput) {
        // TODO(Part 3): move `output` into the transaction.
        // let _ = output;

        self.outputs.push(output);
    }

    pub fn total_input_value(&self) -> u64 {
        // TODO(Part 3): match both InputKind variants and sum their values.

        self.inputs
            .iter()
            .map(|input| match input {
                InputKind::Regular { value, .. } => value,
                InputKind::Coinbase { reward, .. } => reward,
            })
            .sum()
    }

    pub fn total_output_value(&self) -> u64 {
        // TODO(Part 3): sum the value of every output.

        self.outputs.iter().map(|output| output.value).sum()
    }

    pub fn fee(&self) -> Result<u64, TransactionError> {
        // TODO(Part 3): checked subtraction must return OutputsExceedInputs.
        let total_inputs = self.total_input_value();
        let total_outputs = self.total_output_value();

        if total_outputs > total_inputs {
            Err(TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            })
        } else {
            let fee = total_inputs - total_outputs;
            Ok(fee)
        }
    }

    pub fn validate(&self) -> Result<(), TransactionError> {
        // TODO(Part 5): apply every validation rule in ASSIGNMENT.md.
        if self.inputs.is_empty() {
            return Err(TransactionError::NoInputs);
        }

        if self.outputs.is_empty() {
            return Err(TransactionError::NoOutputs);
        }

        for output in &self.outputs {
            if output.value == 0 && output.output_type != OutputType::OpReturn {
                return Err(TransactionError::ZeroValueOutput);
            }
        }

        self.fee()?;

        let coinbase = self
            .inputs
            .iter()
            .filter(|i| matches!(i, InputKind::Coinbase { .. }))
            .count();
        let regular = self
            .inputs
            .iter()
            .filter(|i| matches!(i, InputKind::Regular { .. }))
            .count();

        if coinbase > 0 && regular > 0 {
            return Err(TransactionError::CoinbaseMixedWithRegularInputs);
        }

        if coinbase > 1 {
            return Err(TransactionError::MultipleCoinbaseInputs);
        }

        for input in &self.inputs {
            if let InputKind::Regular {
                previous_output, ..
            } = input
            {
                if previous_output.txid.is_empty() {
                    return Err(TransactionError::InvalidTxid);
                }
            }
        }

        Ok(())
    }
}

impl BitcoinValue for TxOutput {
    fn value(&self) -> u64 {
        // TODO(Part 6)
        self.value
    }
}

impl BitcoinValue for InputKind {
    fn value(&self) -> u64 {
        // TODO(Part 6): both variants carry a value under different names.
        match self {
            InputKind::Regular { value, .. } => *value,
            InputKind::Coinbase { reward, .. } => *reward,
        }
    }
}

pub fn highest_value_output(transaction: &Transaction) -> Option<&TxOutput> {
    // TODO(Part 7): borrow from `transaction`; do not clone.
    // let _ = transaction;

    transaction.outputs.iter().max_by_key(|output| output.value)
}

pub fn find_outputs_for_recipient<'a>(
    transaction: &'a Transaction,
    recipient: &str,
) -> Vec<&'a TxOutput> {
    // TODO(Part 7): return references to all matching outputs.
    // let _ = (transaction, recipient);

    transaction
        .outputs
        .iter()
        .filter(|output| output.recipient == recipient)
        .collect()
}

impl fmt::Display for OutPoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(Part 6): format as `<txid>:<vout>`.
        write!(formatter, "{0}:{1}", self.txid, self.vout)
    }
}

impl fmt::Display for TxOutput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(Part 6)

        write!(
            formatter,
            "Output{{ value: {}, recipient: {}, type = {:?} }}",
            self.value, self.recipient, self.output_type
        )
    }
}

impl fmt::Display for InputKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(Part 6)
        match self {
            InputKind::Regular {
                previous_output,
                value,
                sequence,
            } => {
                write!(
                    formatter,
                    "Regular{{ Outpoint: {}, value: {}, sequence: {} }}",
                    previous_output, value, sequence
                )
            }
            InputKind::Coinbase {
                block_height,
                reward,
            } => {
                write!(
                    formatter,
                    "Coinbase{{ block_height: {}, reward: {} }}",
                    block_height, reward
                )
            }
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(Part 6): print the readable summary described in the assignment.

        let fee = match self.fee() {
            Ok(fee) => format!("{fee}"),
            Err(e) => format!("Invalid fee: {e}"),
        };
        write!(
            formatter, "Transaction {{ version: {}, locktime: {}, inputs: {}, outputs: {}, total_input: {} sats, total_output: {} sats, fee: {} sats }}", self.version, self.locktime, self.inputs.len(), self.outputs.len(), self.total_input_value(), self.total_output_value(), fee
        )
    }
}
