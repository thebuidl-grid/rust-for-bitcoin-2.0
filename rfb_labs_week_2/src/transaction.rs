use std::{fmt, write};

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
        self.inputs.push(input)
    }

    pub fn add_output(&mut self, output: TxOutput) {
        self.outputs.push(output)
    }

    pub fn total_input_value(&self) -> u64 {
        let mut total = 0;

        for input in &self.inputs {
            match input {
                InputKind::Coinbase { reward, .. } => total += reward,
                InputKind::Regular { value, .. } => total += value,
            }
        }

        total
    }

    pub fn total_output_value(&self) -> u64 {
        let mut total_output = 0;

        for output in &self.outputs {
            total_output += output.value
        }

        total_output
    }

    pub fn fee(&self) -> Result<u64, TransactionError> {
        let total_inputs = self.total_input_value();
        let total_outputs = self.total_output_value();
        if total_inputs < total_outputs {
            return Err(TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            });
        }

        let fee = total_inputs - total_outputs;

        Ok(fee)
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

        let mut regular_count = 0;
        let mut coinbase_count = 0;

        for input in &self.inputs {
            match input {
                InputKind::Regular {
                    previous_output, ..
                } => {
                    regular_count += 1;

                    if previous_output.txid.is_empty() {
                        return Err(TransactionError::InvalidTxid);
                    }
                }
                InputKind::Coinbase { .. } => {
                    coinbase_count += 1;
                }
            }
        }

        if regular_count > 0 && coinbase_count > 0 {
            return Err(TransactionError::CoinbaseMixedWithRegularInputs);
        }

        if coinbase_count > 1 {
            return Err(TransactionError::MultipleCoinbaseInputs);
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
            InputKind::Regular { value, .. } => *value,
            InputKind::Coinbase { reward, .. } => *reward,
        }
    }
}

pub fn highest_value_output(transaction: &Transaction) -> Option<&TxOutput> {
    let mut highest_value: Option<&TxOutput> = None;
    for output in &transaction.outputs {
        match highest_value {
            None => highest_value = Some(output),
            Some(val) => {
                if output.value > val.value {
                    highest_value = Some(output)
                }
            }
        }
    }

    highest_value
}

pub fn find_outputs_for_recipient<'a>(
    transaction: &'a Transaction,
    recipient: &str,
) -> Vec<&'a TxOutput> {
    let mut recipients_transactions = Vec::new();
    for txn in &transaction.outputs {
        if txn.recipient == recipient {
            recipients_transactions.push(txn);
        }
    }

    recipients_transactions
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
            InputKind::Regular {
                previous_output,
                value,
                sequence,
            } => write!(
                formatter,
                "regular input: {value} sats from {previous_output}, sequence {sequence}"
            ),
            InputKind::Coinbase {
                block_height,
                reward,
            } => write!(
                formatter,
                "coinbase input: {reward} sats at block {block_height}"
            ),
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let input_count = self.inputs.len();
        let output_count = self.outputs.len();
        let total_inputs = self.total_input_value();
        let total_outputs = self.total_output_value();

        match self.fee(){
         Ok(fee) => write!(formatter, "Transaction(version={}, locktime={}, inputs={}, outputs={}, total input={} sats, total output={} sats, fee={} sats)",
                self.version,
                self.locktime,
                input_count,
                output_count,
                total_inputs,
                total_outputs,
                fee),

         Err(error) => write!(
                formatter,
                "Transaction(version={}, locktime={}, inputs={}, outputs={}, total input={} sats, total output={} sats, fee=invalid: {})",
                self.version,
                self.locktime,
                input_count,
                output_count,
                total_inputs,
                total_outputs,
                error)
        }
    }
}
