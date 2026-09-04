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
        self.inputs.push(input);
    }

    pub fn add_output(&mut self, output: TxOutput) {
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

        let coinbase_inputs = self
            .inputs
            .iter()
            .filter(|input| matches!(input, InputKind::Coinbase { .. }))
            .count();
        let regular_inputs = self.inputs.len() - coinbase_inputs;

        if coinbase_inputs > 1 {
            return Err(TransactionError::MultipleCoinbaseInputs);
        }

        if coinbase_inputs > 0 && regular_inputs > 0 {
            return Err(TransactionError::CoinbaseMixedWithRegularInputs);
        }

        if self.inputs.iter().any(|input| {
            matches!(
                input,
                InputKind::Regular {
                    previous_output: OutPoint { txid, .. },
                    ..
                } if txid.is_empty()
            )
        }) {
            return Err(TransactionError::InvalidTxid);
        }

        self.fee()?;
        Ok(())
    }
}

impl BitcoinValue for TxOutput {
    fn value(&self) -> u64 {
        self.value
    }
}

impl BitcoinValue for InputKind {
    fn value(&self) -> u64 {
        match self {
            Self::Regular { value, .. } => *value,
            Self::Coinbase { reward, .. } => *reward,
        }
    }
}

pub fn highest_value_output(transaction: &Transaction) -> Option<&TxOutput> {
    transaction.outputs.iter().max_by_key(|output| output.value)
}

pub fn find_outputs_for_recipient<'a>(
    transaction: &'a Transaction,
    recipient: &str,
) -> Vec<&'a TxOutput> {
    transaction
        .outputs
        .iter()
        .filter(|output| output.recipient == recipient)
        .collect()
}

impl fmt::Display for OutPoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.txid, self.vout)
    }
}

impl fmt::Display for TxOutput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} sats to {} ({:?})",
            self.value, self.recipient, self.output_type
        )
    }
}

impl fmt::Display for InputKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Regular {
                previous_output,
                value,
                sequence,
            } => write!(
                formatter,
                "regular input: {previous_output}, {value} sats, sequence {sequence}"
            ),
            Self::Coinbase {
                block_height,
                reward,
            } => write!(
                formatter,
                "coinbase input: block {block_height}, {reward} sats"
            ),
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fee = match self.fee() {
            Ok(fee) => format!("{fee} sats"),
            Err(error) => format!("invalid ({error})"),
        };

        write!(
            formatter,
            "Transaction v{} (locktime {}): {} input(s), {} output(s), inputs: {} sats, outputs: {} sats, fee: {fee}",
            self.version,
            self.locktime,
            self.inputs.len(),
            self.outputs.len(),
            self.total_input_value(),
            self.total_output_value(),
        )
    }
}
