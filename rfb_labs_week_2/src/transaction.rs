use std::fmt;

use crate::error::TransactionError;
use crate::state::TransactionState;

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

pub trait BitcoinValue {
    fn value(&self) -> u64;

    fn value_in_btc(&self) -> f64 {
        self.value() as f64 / 100_000_000.0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Transaction {
    pub version: i32,
    pub inputs: Vec<InputKind>,
    pub outputs: Vec<TxOutput>,
    pub locktime: u32,
    pub state: TransactionState,
}

impl Transaction {
    pub fn new(version: i32, locktime: u32) -> Self {
        Self {
            version,
            inputs: Vec::new(),
            outputs: Vec::new(),
            locktime,
            state: TransactionState::Created,
        }
    }

    pub fn add_input(&mut self, input: InputKind) {
        // TODO(Part 3): move `input` into the transaction.
        // todo!("add an input")
        self.inputs.push(input);
    }

    pub fn add_output(&mut self, output: TxOutput) {
        // TODO(Part 3): move `output` into the transaction.
        // todo!("add an output")
        self.outputs.push(output);
    }

    pub fn total_input_value(&self) -> u64 {
        self.inputs.iter().fold(0, |acc, input| match input {
            InputKind::Regular { value, .. } => acc + value,
            InputKind::Coinbase { reward, .. } => acc + reward,
        })
    }

    pub fn total_output_value(&self) -> u64 {
        // TODO(Part 3): sum the value of every output.
        // todo!("calculate the total output value")
        self.outputs
            .iter()
            .fold(0, |acc, output| acc + output.value)
    }

    pub fn fee(&self) -> Result<u64, TransactionError> {
        // TODO(Part 3): checked subtraction must return OutputsExceedInputs.
        // todo!("calculate the fee")
        let fee = self
            .total_input_value()
            .checked_sub(self.total_output_value());
        match fee {
            Some(fee) => Ok(fee),
            None => Err(TransactionError::OutputsExceedInputs {
                total_inputs: self.total_input_value(),
                total_outputs: self.total_output_value(),
            }),
        }
    }

    pub fn validate(&self) -> Result<(), TransactionError> {
        // TODO(Part 5): apply every validation rule in ASSIGNMENT.md.
        // todo!("validate the transaction")
        if self.inputs.is_empty() && !self.outputs.is_empty() {
            return Err(TransactionError::NoInputs);
        }

        if !self.inputs.is_empty() && self.outputs.is_empty() {
            return Err(TransactionError::NoOutputs);
        }

        // non-OpReturn zero outputs, OutputType is an enum with a variant of OpReturn
        let non_op_return = self
            .outputs
            .iter()
            .filter(|output| output.output_type != OutputType::OpReturn)
            .all(|output| output.value != 0);

        if !non_op_return {
            return Err(TransactionError::ZeroValueOutput);
        };

        // empty regular input txns
        for input in &self.inputs {
            if let InputKind::Regular {
                previous_output, ..
            } = input
            {
                if previous_output.txid.is_empty() {
                    return Err(TransactionError::InvalidTxid);
                }
            };
        }

        // mixed coinbase/regular inputs
        let has_regular_inputs = self
            .inputs
            .iter()
            .any(|input| matches!(input, InputKind::Regular { .. }));

        let has_coinbase_inputs = self
            .inputs
            .iter()
            .any(|input| matches!(input, InputKind::Coinbase { .. }));

        if has_regular_inputs && has_coinbase_inputs {
            return Err(TransactionError::CoinbaseMixedWithRegularInputs);
        }

        // outputs exceeding inputs
        self.total_input_value()
            .checked_sub(self.total_output_value())
            .ok_or(TransactionError::OutputsExceedInputs {
                total_inputs: self.total_input_value(),
                total_outputs: self.total_output_value(),
            })?;

        // multiple coinbase inputs
        let coinbase_input_count = self
            .inputs
            .iter()
            .filter(|input| matches!(input, InputKind::Coinbase { .. }))
            .count()
            > 1;

        if coinbase_input_count {
            return Err(TransactionError::MultipleCoinbaseInputs);
        }

        Ok(())
    }

    fn transition(
        &mut self,
        required: TransactionState,
        next: TransactionState,
    ) -> Result<(), TransactionError> {
        if self.state != required {
            return Err(TransactionError::InvalidStateTransition {
                from: self.state,
                to: next,
            });
        }
        self.state = next;
        Ok(())
    }

    pub fn mark_validated(&mut self) -> Result<(), TransactionError> {
        self.transition(TransactionState::Created, TransactionState::Validated)
    }

    pub fn sign(&mut self) -> Result<(), TransactionError> {
        self.transition(TransactionState::Validated, TransactionState::Signed)
    }

    pub fn broadcast(&mut self) -> Result<(), TransactionError> {
        self.transition(TransactionState::Signed, TransactionState::Broadcast)
    }

    pub fn confirm(&mut self) -> Result<(), TransactionError> {
        self.transition(TransactionState::Broadcast, TransactionState::Confirmed)
    }

    pub fn reject(&mut self) -> Result<(), TransactionError> {
        if matches!(
            self.state,
            TransactionState::Confirmed | TransactionState::Rejected
        ) {
            return Err(TransactionError::InvalidStateTransition {
                from: self.state,
                to: TransactionState::Rejected,
            });
        }
        self.state = TransactionState::Rejected;
        Ok(())
    }
}

impl BitcoinValue for TxOutput {
    fn value(&self) -> u64 {
        // TODO(Part 6)
        // todo!("return the output value")
        self.value
    }
}

impl BitcoinValue for InputKind {
    fn value(&self) -> u64 {
        // TODO(Part 6): both variants carry a value under different names.
        // todo!("return the input value")
        match self {
            InputKind::Regular { value, .. } => *value,
            InputKind::Coinbase { reward, .. } => *reward,
        }
    }
}

pub fn highest_value_output(transaction: &Transaction) -> Option<&TxOutput> {
    // TODO(Part 7): borrow from `transaction`; do not clone.
    // let _ = transaction;
    // todo!("find the highest-value output")
    let output = transaction
        .outputs
        .iter()
        .max_by(|output1, output2| output1.value().cmp(&output2.value()))?;

    Some(output)
}

pub fn find_outputs_for_recipient<'a>(
    transaction: &'a Transaction,
    recipient: &str,
) -> Vec<&'a TxOutput> {
    // TODO(Part 7): return references to all matching outputs.
    // let _ = (*transaction, *recipient);
    // todo!("find outputs for a recipient")
    transaction
        .outputs
        .iter()
        .filter(|output| output.recipient == recipient)
        .collect::<Vec<&TxOutput>>()
}

impl fmt::Display for OutPoint {
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(Part 6): format as `<txid>:<vout>`.
        // todo!("display an outpoint")
        write!(_formatter, "{}:{}", self.txid, self.vout)
    }
}

impl fmt::Display for TxOutput {
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(Part 6)
        // todo!("display an output")
        write!(_formatter, "{:?}", self)
    }
}

impl fmt::Display for InputKind {
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(Part 6)
        // todo!("display an input")
        write!(_formatter, "{:?}", self)
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO(Part 6): print the readable summary described in the assignment.
        // todo!("display a transaction summary")
        let response = format!(
            "\
           Transaction Summary\n\
           version: {}\n\
           locktime: {}\n\
           inputs:{}\n\
           outputs:{}\n\
           total_output_value: {}\n\
           total_input_value: {}\n\
        ",
            self.version,
            self.locktime,
            self.inputs.len(),
            self.outputs.len(),
            self.total_output_value(),
            self.total_input_value()
        );
        write!(_formatter, "{}", response)
    }
}
