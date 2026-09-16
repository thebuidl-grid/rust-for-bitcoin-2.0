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
        // TODO (part 3): move input into the transaction.
        // let_ = input;
        // todo!("add an input")    

        self.inputs.push(input);
    }

    pub fn add_output(&mut self, output: TxOutput) {
        // TODO(part 3): move output into the transaction.
        // let_ = output;
        // todo!("add an output")

        self.outputs.push(output);
    }

    pub fn total_input_value(&self) -> u64 {
        // TODO(part 3): match both inputkind variants and sum their values.
        // todo!("calculate the total input value")

        self.inputs.iter().map(|input| input.value()).sum()
    }

    pub fn total_output_value(&self) -> u64 {
        // TODO(Part 3): sum the value of every output.
        // todo!("calculate the total output value")

        self.outputs.iter().map(|output| output.value()).sum()
    }

    pub fn fee(&self) -> Result<u64, TransactionError> {
         // TODO(Part 3): checked subtraction must return OutputsExceedInputs.
        // todo!("calculate the fee")
        let inputs = self.total_input_value();
        let outputs = self.total_output_value();
        inputs
            .checked_sub(outputs)
            .ok_or(TransactionError::OutputsExceedInputs {
                total_inputs: inputs,
                total_outputs: outputs,
            })
    }

    pub fn validate(&self) -> Result<(), TransactionError> {
        //TODO(Part 5): apply every validation rule in ASSIGNMENT.md.
        // todo!("validate the transaction")
        // No inputs
        if self.inputs.is_empty() {
            return Err(TransactionError::NoInputs);
        }

        // No outputs
        if self.outputs.is_empty() {
            return Err(TransactionError::NoOutputs);
        }

        // Check for mixed coinbase and regular inputs
        let has_coinbase = self
            .inputs
            .iter()
            .any(|input| matches!(input, InputKind::Coinbase { .. }));
        let has_regular = self
            .inputs
            .iter()
            .any(|input| matches!(input, InputKind::Regular { .. }));

        if has_coinbase && has_regular {
            return Err(TransactionError::CoinbaseMixedWithRegularInputs);
        }

        // Multiple coinbase inputs
        let coinbase_count = self
            .inputs
            .iter()
            .filter(|input| matches!(input, InputKind::Coinbase { .. }))
            .count();
        if coinbase_count > 1 {
            return Err(TransactionError::MultipleCoinbaseInputs);
        }

        // Empty TXIDs in regular inputs
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

        // Non-OP_RETURN zero outputs
        for output in &self.outputs {
            if output.value == 0 && output.output_type != OutputType::OpReturn {
                return Err(TransactionError::ZeroValueOutput);
            }
        }

        // Outputs exceeding inputs
        let _ = self.fee()?;

        Ok(())
    }
}

impl BitcoinValue for TxOutput {
    fn value(&self) -> u64 {
        // Todo(part 6)
        // todo!("return the output value")
        self.value
    }
}

impl BitcoinValue for InputKind {
    fn value(&self) -> u64 {
        // Todo(part 6): both variant carry a value under different names.
        // todo!("return the input value")
        match self {
            InputKind::Regular { value, .. } => *value,
            InputKind::Coinbase { reward, .. } => *reward,
        }
    }
}

pub fn highest_value_output(transaction: &Transaction) -> Option<&TxOutput> {
    // Todo(part 7): borrow from transaction; do not clone.
    let _ = transaction;
    // todo!("find the hihest-value output")

    transaction.outputs.iter().max_by_key(|output| output.value)
}

pub fn find_outputs_for_recipient<'a>(
    transaction: &'a Transaction,
    recipient: &str,
) -> Vec<&'a TxOutput> {
    // Todo(part 7): return references to all matching outputs.
    let _ = (transaction, recipient);
    // todo!("find outputs for a recipient")

    transaction
        .outputs
        .iter()
        .filter(|output| output.recipient == recipient)
        .collect()
}

impl fmt::Display for OutPoint {
    // fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {}
    fn fmt(&self, formatter: &mut fmt:: Formatter<'_>) -> fmt::Result {
        // // TODO(part 6): format as <txid>:<vout>.
        // todo!("display an OutPoint")

        write!(formatter, "{}:{}", self.txid, self.vout)
    }
}

impl fmt::Display for TxOutput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(part 6)
        // todo!("display an OutPoint")

        write!(
            formatter,
            "Output{{ value: {}, recipient: {}, type = {:?} }}",
            self.value, self.recipient, self.output_type
        )
    }
}

impl fmt::Display for InputKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(part 6)
        // todo!("display an input")

        match self {
            InputKind::Regular {
                previous_output,
                value,
                sequence,
            } => {
                write!(
                    formatter,
                    "Regular input: {} sats from {} (sequence: {})",
                    value, previous_output, sequence
                )
            }
            InputKind::Coinbase {
                block_height,
                reward,
            } => {
                write!(
                    formatter,
                    "Coinbase input: {} sats from block {}",
                    reward, block_height
                )
            }
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
          // TODO(part 6)  print the readable summary described in the assignment.
        // todo!("display a transaction summary")

        let total_input = self.total_input_value();
        let total_output = self.total_output_value();
        let fee = self.fee().unwrap_or(0);

        writeln!(formatter, "Bitcoin Transaction Summary")?;
        writeln!(formatter, "==============================")?;
        writeln!(formatter, "Version: {}", self.version)?;
        writeln!(formatter, "Locktime: {}", self.locktime)?;
        writeln!(
            formatter,
            "Inputs: {} (total: {} sats)",
            self.inputs.len(),
            total_input
        )?;
        writeln!(
            formatter,
            "Outputs: {} (total: {} sats)",
            self.outputs.len(),
            total_output
        )?;

        if let Err(e) = self.fee() {
            writeln!(formatter, "Fee: INVALID ({})", e)?;
        } else {
            writeln!(formatter, "Fee: {} sats", fee)?;
        }

        Ok(())
    }
}
