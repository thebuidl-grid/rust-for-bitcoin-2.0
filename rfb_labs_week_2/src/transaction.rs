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

    /// Takes ownership of `input`; the caller can no longer use the value.
    pub fn add_input(&mut self, input: InputKind) {
        self.inputs.push(input);
    }

    /// Takes ownership of `output`; the caller can no longer use the value.
    pub fn add_output(&mut self, output: TxOutput) {
        self.outputs.push(output);
    }

    /// Sums both input variants through `BitcoinValue`, so the per-variant
    /// match lives in exactly one place.
    pub fn total_input_value(&self) -> u64 {
        self.inputs
            .iter()
            .fold(0u64, |total, input| total.saturating_add(input.value()))
    }

    pub fn total_output_value(&self) -> u64 {
        self.outputs
            .iter()
            .fold(0u64, |total, output| total.saturating_add(output.value()))
    }

    /// Fee is whatever the inputs provide that the outputs do not claim.
    /// Subtracting is checked so an over-spending transaction reports an error
    /// instead of wrapping around to a huge fee.
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

    /// Applies every rule from the assignment, in the order it lists them.
    pub fn validate(&self) -> Result<(), TransactionError> {
        if self.inputs.is_empty() {
            return Err(TransactionError::NoInputs);
        }

        if self.outputs.is_empty() {
            return Err(TransactionError::NoOutputs);
        }

        // A zero-value output is only meaningful for OP_RETURN, which carries
        // data rather than coins.
        if self
            .outputs
            .iter()
            .any(|output| output.value == 0 && output.output_type != OutputType::OpReturn)
        {
            return Err(TransactionError::ZeroValueOutput);
        }

        // `?` propagates OutputsExceedInputs; the fee itself is not needed here.
        self.fee()?;

        let coinbase_count = self
            .inputs
            .iter()
            .filter(|input| matches!(input, InputKind::Coinbase { .. }))
            .count();

        if coinbase_count > 0 && coinbase_count < self.inputs.len() {
            return Err(TransactionError::CoinbaseMixedWithRegularInputs);
        }

        if coinbase_count > 1 {
            return Err(TransactionError::MultipleCoinbaseInputs);
        }

        if self.inputs.iter().any(|input| {
            matches!(input, InputKind::Regular { previous_output, .. }
                if previous_output.txid.trim().is_empty())
        }) {
            return Err(TransactionError::InvalidTxid);
        }

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

impl fmt::Display for OutputType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            OutputType::P2pkh => "P2PKH",
            OutputType::P2wpkh => "P2WPKH",
            OutputType::P2tr => "P2TR",
            OutputType::OpReturn => "OP_RETURN",
        };

        formatter.write_str(label)
    }
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
            "{} sats ({:.8} BTC) to {} [{}]",
            self.value,
            self.value_in_btc(),
            self.recipient,
            self.output_type
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
                "regular {} worth {} sats ({:.8} BTC), sequence {}",
                previous_output,
                value,
                self.value_in_btc(),
                sequence
            ),
            InputKind::Coinbase {
                block_height,
                reward,
            } => write!(
                formatter,
                "coinbase at height {} worth {} sats ({:.8} BTC)",
                block_height,
                reward,
                self.value_in_btc()
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
            "  {} input(s) totalling {} sats",
            self.inputs.len(),
            self.total_input_value()
        )?;
        for input in &self.inputs {
            writeln!(formatter, "    - {input}")?;
        }

        writeln!(
            formatter,
            "  {} output(s) totalling {} sats",
            self.outputs.len(),
            self.total_output_value()
        )?;
        for output in &self.outputs {
            writeln!(formatter, "    - {output}")?;
        }

        // An over-spending transaction has no fee to report, but printing it
        // must never panic.
        match self.fee() {
            Ok(fee) => write!(formatter, "  fee: {fee} sats"),
            Err(error) => write!(formatter, "  fee: unavailable ({error})"),
        }
    }
}
