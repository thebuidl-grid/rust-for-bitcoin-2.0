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
        self.inputs.push(input)
    }

    pub fn add_output(&mut self, output: TxOutput) {
        // TODO(Part 3): move `output` into the transaction.
        self.outputs.push(output);
    }

    pub fn total_input_value(&self) -> u64 {
        self.inputs
            .iter()
            .map(|input| match input {
                InputKind::Regular { value, .. } => *value,
                InputKind::Coinbase { reward, .. } => *reward,
            })
            .sum()
    }

    pub fn total_output_value(&self) -> u64 {
        self.outputs.iter().map(|output| output.value).sum()
    }

    pub fn fee(&self) -> Result<u64, TransactionError> {
        let total_inputs = self.total_input_value();
        let total_outputs = self.total_output_value();

        total_inputs
            .checked_sub(total_outputs)
            .ok_or(TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            })
    }

    pub fn validate(&self) -> Result<(), TransactionError> {
        if self.inputs.is_empty() {
            return Err(TransactionError::NoInputs);
        }

        if self.outputs.is_empty() {
            return Err(TransactionError::NoOutputs);
        }

        if self
            .outputs
            .iter()
            .any(|output| output.value == 0 && output.output_type != OutputType::OpReturn)
        {
            return Err(TransactionError::ZeroValueOutput);
        }

        if self.inputs.iter().any(|input| {
            matches!(
                input,
                InputKind::Regular {
                    previous_output,
                    ..
                } if previous_output.txid.trim().is_empty()
            )
        }) {
            return Err(TransactionError::InvalidTxid);
        }

        let coinbase_count = self
            .inputs
            .iter()
            .filter(|input| matches!(input, InputKind::Coinbase { .. }))
            .count();

        let regular_count = self
            .inputs
            .iter()
            .filter(|input| matches!(input, InputKind::Regular { .. }))
            .count();

        if coinbase_count > 1 {
            return Err(TransactionError::MultipleCoinbaseInputs);
        }

        if coinbase_count > 0 && regular_count > 0 {
            return Err(TransactionError::CoinbaseMixedWithRegularInputs);
        }

        self.fee()?;

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
            InputKind::Regular { value, .. } => value.to_owned(),
            InputKind::Coinbase { reward, .. } => reward.to_owned(),
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
        .filter(|output| output.recipient == *recipient)
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
            "{} sats to {} ({:?})",
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
                    "Regular Input: {} ({} sats), sequence: {}",
                    previous_output, value, sequence
                )
            }

            InputKind::Coinbase {
                block_height,
                reward,
            } => {
                write!(
                    formatter,
                    "Coinbase Input: block height {}, reward {} sats",
                    block_height, reward
                )
            }
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "Transaction")?;
        writeln!(formatter, "Version: {}", self.version)?;
        writeln!(formatter, "Locktime: {}", self.locktime)?;

        writeln!(formatter, "Inputs:")?;
        for input in &self.inputs {
            writeln!(formatter, "  - {}", input)?;
        }

        writeln!(formatter, "Outputs:")?;
        for output in &self.outputs {
            writeln!(formatter, "  - {}", output)?;
        }

        writeln!(formatter, "Total input: {} sats", self.total_input_value())?;
        writeln!(
            formatter,
            "Total output: {} sats",
            self.total_output_value()
        )?;

        match self.fee() {
            Ok(fee) => writeln!(formatter, "Fee: {} sats", fee),
            Err(error) => writeln!(formatter, "Fee: error ({})", error),
        }
    }
}
