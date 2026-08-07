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
        self.inputs.push(input);
    }

    pub fn add_output(&mut self, output: TxOutput) {
        // TODO(Part 3): move `output` into the transaction.
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
       self.outputs.iter().map(|output| output.value).sum()
    }

    pub fn fee(&self) -> Result<u64, TransactionError> {
        // TODO(Part 3): checked subtraction must return OutputsExceedInputs.
        let total_inputs = self.total_input_value();
        let total_outputs = self.total_output_value();

        total_inputs.checked_sub(total_outputs).ok_or(
            TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
        },
    )
    }

    pub fn validate(&self) -> Result<(), TransactionError> {
        // TODO(Part 5): apply every validation rule in ASSIGNMENT.md.
       //have at least one input.
    if self.inputs.is_empty() {
        return Err(TransactionError::NoInputs);
    }

    // at least one output.
    if self.outputs.is_empty() {
        return Err(TransactionError::NoOutputs);
    }

    // Zero-value outputs are only allowed for OP_RETURN.
    for output in &self.outputs {
        if output.value == 0 && output.output_type != OutputType::OpReturn {
            return Err(TransactionError::ZeroValueOutput);
        }
    }

    let mut coinbase_count = 0;

    for input in &self.inputs {
        match input {
            InputKind::Coinbase { .. } => {
                coinbase_count += 1;
            }

            InputKind::Regular { previous_output, .. } => {
                if previous_output.txid.is_empty() {
                    return Err(TransactionError::InvalidTxid);
                }
            }
        }
    }

    // Only one coinbase input is allowed.
    if coinbase_count > 1 {
        return Err(TransactionError::MultipleCoinbaseInputs);
    }

    // Coinbase transactions cannot contain regular inputs.
    if coinbase_count == 1 && self.inputs.len() > 1 {
        return Err(TransactionError::CoinbaseMixedWithRegularInputs);
    }

    // Outputs cannot exceed inputs.
    if self.total_output_value() > self.total_input_value() {
        return Err(TransactionError::OutputsExceedInputs {
            total_inputs: self.total_input_value(),
            total_outputs: self.total_output_value(),
        });
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
            "{} sats -> {} ({:?})",
            self.value,
            self.recipient,
            self.output_type
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
            } => write!(
                formatter,
                "Regular({}, {} sats, seq={})",
                previous_output,
                value,
                sequence
            ),

            InputKind::Coinbase {
                block_height,
                reward,
            } => write!(
                formatter,
                "Coinbase(height={}, reward={} sats)",
                block_height,
                reward
            ),
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(Part 6): print the readable summary described in the assignment.
         writeln!(formatter, "Transaction")?;
        writeln!(formatter, "  Version: {}", self.version)?;
        writeln!(formatter, "  Locktime: {}", self.locktime)?;
        writeln!(formatter, "  Inputs: {}", self.inputs.len())?;
        writeln!(formatter, "  Outputs: {}", self.outputs.len())?;
        writeln!(
            formatter,
            "  Total Input: {} sats",
            self.total_input_value()
        )?;
        writeln!(
            formatter,
            "  Total Output: {} sats",
            self.total_output_value()
        )?;

        match self.fee() {
            Ok(fee) => writeln!(formatter, "  Fee: {} sats", fee)?,
            Err(err) => writeln!(formatter, "  Fee: {}", err)?,
        }

        Ok(())
    }
}
