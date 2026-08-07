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
        // Each input is moved here so the caller can no longer use it.
        self.inputs.push(input);
    }

    pub fn add_output(&mut self, output: TxOutput) {
        // The output is moved in and owned by the transaction from now on.
        self.outputs.push(output);
    }

    pub fn total_input_value(&self) -> u64 {
        // Both variants carry a value, just under different field names, so we
        // delegate to the BitcoinValue impl to keep this branch-free.
        self.inputs.iter().map(BitcoinValue::value).sum()
    }

    pub fn total_output_value(&self) -> u64 {
        self.outputs.iter().map(TxOutput::value).sum()
    }

    pub fn fee(&self) -> Result<u64, TransactionError> {
        let total_inputs = self.total_input_value();
        let total_outputs = self.total_output_value();
        // checked_sub returns None on underflow instead of panicking, which we
        // surface as an explicit domain error rather than wrapping the value.
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

        let mut coinbase_count = 0u32;
        for input in &self.inputs {
            match input {
                InputKind::Coinbase { .. } => coinbase_count += 1,
                InputKind::Regular {
                    previous_output, ..
                } => {
                    if previous_output.txid.is_empty() {
                        return Err(TransactionError::InvalidTxid);
                    }
                }
            }
        }

        // A coinbase is the first input of a block, so mixing it with regular
        // inputs makes the transaction impossible to interpret correctly.
        if coinbase_count > 1 {
            return Err(TransactionError::MultipleCoinbaseInputs);
        }
        if coinbase_count == 1 && self.inputs.len() > 1 {
            return Err(TransactionError::CoinbaseMixedWithRegularInputs);
        }

        // Only OP_RETURN allows a zero-value output; a normal payment must
        // carry a non-zero amount to be worth confirming.
        for output in &self.outputs {
            if output.value == 0 && output.output_type != OutputType::OpReturn {
                return Err(TransactionError::ZeroValueOutput);
            }
        }

        // A valid transaction can never pay out more than it brings in.
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
        // Regular and coinbase inputs store their value under different names.
        match self {
            InputKind::Regular { value, .. } => *value,
            InputKind::Coinbase { reward, .. } => *reward,
        }
    }
}

pub fn highest_value_output(transaction: &Transaction) -> Option<&TxOutput> {
    // max_by_key hands back a borrowed element; no clones are made.
    transaction.outputs.iter().max_by_key(|output| output.value)
}

pub fn find_outputs_for_recipient<'a>(
    transaction: &'a Transaction,
    recipient: &str,
) -> Vec<&'a TxOutput> {
    // The returned references borrow from the transaction so the caller keeps
    // owning the actual outputs.
    transaction
        .outputs
        .iter()
        .filter(|output| output.recipient == recipient)
        .collect()
}

impl fmt::Display for OutPoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // outpoints are conventionally rendered as "<txid>:<vout>".
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
                "regular input spending {previous_output}, {value} sats, sequence {sequence}"
            ),
            InputKind::Coinbase {
                block_height,
                reward,
            } => write!(
                formatter,
                "coinbase input for block {block_height}, {reward} sats"
            ),
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            formatter,
            "Transaction v{} (locktime {})",
            self.version, self.locktime
        )?;
        writeln!(
            formatter,
            "  inputs:  {} total {} sats",
            self.inputs.len(),
            self.total_input_value()
        )?;
        writeln!(
            formatter,
            "  outputs: {} total {} sats",
            self.outputs.len(),
            self.total_output_value()
        )?;
        match self.fee() {
            Ok(fee) => writeln!(formatter, "  fee:     {fee} sats"),
            Err(error) => writeln!(formatter, "  fee:     invalid ({error})"),
        }
    }
}
