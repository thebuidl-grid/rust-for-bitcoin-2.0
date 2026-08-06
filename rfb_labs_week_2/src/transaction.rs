use std::fmt;

use crate::error;

use crate::InputKind::{Coinbase, Regular};
use error::TransactionError;

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
        let _ = input;
        // todo!("add an input")
        self.inputs.push(input);
    }

    pub fn add_output(&mut self, output: TxOutput) {
        // TODO(Part 3): move `output` into the transaction.
        // todo!("add an output")
        self.outputs.push(output);
    }

    pub fn total_input_value(&self) -> u64 {
        // TODO(Part 3): match both InputKind variants and sum their values.
        // todo!("calculate the total input value")
        let value = self
            .inputs
            .iter()
            .map(|element| match element {
                Coinbase { reward, .. } => reward,
                Regular { value, .. } => value,
            })
            .sum();

        value
    }

    pub fn total_output_value(&self) -> u64 {
        // TODO(Part 3): sum the value of every output.
        // todo!("calculate the total output value")
        let total_output = self.outputs.iter().map(|element| element.value).sum();

        total_output
    }

    pub fn fee(&self) -> Result<u64, TransactionError> {
        // TODO(Part 3): checked subtraction must return OutputsExceedInputs.
        // todo!("calculate the fee")

        let total_output = self.total_output_value();
        let total_input = self.total_input_value();

        total_input
            .checked_sub(total_output)
            .ok_or(TransactionError::OutputsExceedInputs {
                total_inputs: total_input,
                total_outputs: total_output,
            })
    }

    pub fn validate(&self) -> Result<(), TransactionError> {
        // TODO(Part 5): apply every validation rule in ASSIGNMENT.md.
        // todo!("validate the transaction")

        // Rule 1: no inputs
        if self.inputs.is_empty() {
            return Err(TransactionError::NoInputs);
        }

        // Rule 2: no outputs
        if self.outputs.is_empty() {
            return Err(TransactionError::NoOutputs);
        }

        // Rule 3: zero-value non-OpReturn outputs
        for output in &self.outputs {
            if output.value == 0 && output.output_type != OutputType::OpReturn {
                return Err(TransactionError::ZeroValueOutput);
            }
        }

        // Rule 4: outputs exceed inputs
        self.fee()?;

        // Rules 5 & 6: coinbase mixing and multiple coinbase inputs
        let coinbase_count = self
            .inputs
            .iter()
            .filter(|i| matches!(i, InputKind::Coinbase { .. }))
            .count();
        let regular_count = self
            .inputs
            .iter()
            .filter(|i| matches!(i, InputKind::Regular { .. }))
            .count();

        if coinbase_count > 0 && regular_count > 0 {
            return Err(TransactionError::CoinbaseMixedWithRegularInputs);
        }

        if coinbase_count > 1 {
            return Err(TransactionError::MultipleCoinbaseInputs);
        }

        // Rule 7: empty regular-input TXIDs
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
    transaction.outputs.iter().max_by_key(|output| output.value)
}

pub fn find_outputs_for_recipient<'a>(
    transaction: &'a Transaction,
    recipient: &str,
) -> Vec<&'a TxOutput> {
    // TODO(Part 7): return references to all matching outputs.
    transaction
        .outputs
        .iter()
        .filter(|output| output.recipient == recipient)
        .collect()
}

impl fmt::Display for OutPoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(Part 6): format as `<txid>:<vout>`.
        write!(formatter, "{}:{}", self.txid, self.vout)
    }
}

impl fmt::Display for TxOutput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(Part 6)
        write!(
            formatter,
            "Output {{ value: {} sats, recipient: {}, type: {:?} }}",
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
                    "Regular {{ outpoint: {previous_output}, value: {value} sats, sequence: {sequence} }}"
                )
            }
            InputKind::Coinbase {
                block_height,
                reward,
            } => {
                write!(
                    formatter,
                    "Coinbase {{ block_height: {block_height}, reward: {reward} sats }}"
                )
            }
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(Part 6): print the readable summary described in the assignment.
        let fee_display = match self.fee() {
            Ok(fee) => format!("{fee} sats"),
            Err(e) => format!("invalid ({e})"),
        };
        write!(
            formatter,
            "Transaction {{ version: {}, locktime: {}, inputs: {}, outputs: {}, total_input: {} sats, total_output: {} sats, fee: {} }}",
            self.version,
            self.locktime,
            self.inputs.len(),
            self.outputs.len(),
            self.total_input_value(),
            self.total_output_value(),
            fee_display,
        )
    }
}
