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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    Created,
    Validated,
    Signed,
    Broadcast,
    Confirmed,
    Rejected,
}

pub trait BitcoinValue {
    fn value(&self) -> u64;

    fn value_in_btc(&self) -> f64 {
        self.value() as f64 / 100_000_000.0
    }
}

impl TransactionState {
    pub fn can_transition_to(self, next: TransactionState) -> bool {
        use TransactionState::*;
        matches!(
            (self, next),
            (Created, Validated)
                | (Created, Rejected)
                | (Validated, Signed)
                | (Validated, Rejected)
                | (Signed, Broadcast)
                | (Signed, Rejected)
                | (Broadcast, Confirmed)
                | (Broadcast, Rejected)
        )
    }

    pub fn transition_to(&mut self, next: TransactionState) -> Result<(), TransactionError> {
        if self.can_transition_to(next) {
            *self = next;
            Ok(())
        } else {
            Err(TransactionError::InvalidStateTransition {
                from: *self,
                to: next,
            })
        }
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
        // move `input` into the transaction.
        self.inputs.push(input);
    }

    pub fn add_output(&mut self, output: TxOutput) {
        self.outputs.push(output);
    }

    pub fn total_input_value(&self) -> u64 {
        // TODO(Part 3): match both InputKind variants and sum their values.
        self.inputs
            .iter()
            .map(|input| match input {
                InputKind::Regular { value, .. } => *value,
                InputKind::Coinbase { reward, .. } => *reward,
            })
            .sum()
    }

    pub fn total_output_value(&self) -> u64 {
        // TODO(Part 3): sum the value of every output.
        self.outputs
            .iter()
            .map(|output| output.value())
            .sum()
    }

    pub fn fee(&self) -> Result<u64, TransactionError> {
        let input_total = self.total_input_value();
        let output_total = self.total_output_value();

        input_total
            .checked_sub(output_total)
            .ok_or(TransactionError::OutputsExceedInputs {
                total_inputs: input_total,
                total_outputs: output_total,
            })
    }

    pub fn validate(&self) -> Result<(), TransactionError> {
        if self.inputs.is_empty() {
            return Err(TransactionError::NoInputs);
        }

        if self.outputs.is_empty() {
            return Err(TransactionError::NoOutputs);
        }

        let mut coinbase_count = 0;
        let mut regular_count = 0;

        for input in &self.inputs {
            match input {
                InputKind::Regular { previous_output, .. } => {
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

        if coinbase_count > 1 {
            return Err(TransactionError::MultipleCoinbaseInputs);
        }

        if coinbase_count > 0 && regular_count > 0 {
            return Err(TransactionError::CoinbaseMixedWithRegularInputs);
        }

        for output in &self.outputs {
            if output.value == 0 && output.output_type != OutputType::OpReturn {
                return Err(TransactionError::ZeroValueOutput);
            }
        }

        self.fee()?; // Check if outputs exceed inputs.
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
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(_formatter, "{}:{}", self.txid, self.vout)
    }
}

impl fmt::Display for TxOutput {
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            _formatter,
            "TxOutput(value: {}, recipient: {}, type: {:?})",
            self.value, self.recipient, self.output_type
        )
    }
}

impl fmt::Display for InputKind {
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputKind::Regular {
                previous_output,
                value,
                sequence,
            } => write!(
                _formatter,
                "RegularInput(previous_output: {}, value: {}, sequence: {})",
                previous_output, value, sequence
            ),
            InputKind::Coinbase { 
                block_height, 
                reward 
            } => write!(
                _formatter,
                "CoinbaseInput(block_height: {}, reward: {})",
                block_height, reward
            ),
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            _formatter,
            "Transaction v{} (locktime {}): {} input(s), {} output(s), total_in={} sats, total_out={} sats, fee=",
            self.version,
            self.locktime,
            self.inputs.len(),
            self.outputs.len(),
            self.total_input_value(),
            self.total_output_value()
        )?;


        match self.fee() {
            Ok(fee) => write!(_formatter, "{} sats", fee),
            Err(e) => write!(_formatter, "Error calculating fee: {}", e),
        }
    }
}
