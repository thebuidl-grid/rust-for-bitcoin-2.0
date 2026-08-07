use std::fmt::{self};

use crate::error::{StateTransitionError, TransactionError};

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
    pub state: TransactionState,
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
            state: TransactionState::Created,
        }
    }

    pub fn add_input(&mut self, input: InputKind) {
        self.inputs.push(input);
    }

    pub fn add_output(&mut self, output: TxOutput) {
        self.outputs.push(output);
    }

    pub fn total_input_value(&self) -> u64 {
        self.inputs.iter().map(|x| x.value()).sum()
    }

    pub fn total_output_value(&self) -> u64 {
        self.outputs.iter().map(|x| x.value).sum()
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

        for output in &self.outputs {
            if output.value == 0 && output.output_type != OutputType::OpReturn {
                return Err(TransactionError::ZeroValueOutput);
            }
        }

        let total_inputs = self.total_input_value();
        let total_outputs = self.total_output_value();
        if total_outputs > total_inputs {
            return Err(TransactionError::OutputsExceedInputs {
                total_inputs,
                total_outputs,
            });
        }

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

        let coinbase_count = self
            .inputs
            .iter()
            .filter(|input| matches!(input, InputKind::Coinbase { .. }))
            .count();
        if coinbase_count > 1 {
            return Err(TransactionError::MultipleCoinbaseInputs);
        }

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

    pub fn current_state(&self) -> TransactionState {
        self.state
    }

    fn can_transition(from: TransactionState, to: TransactionState) -> bool {
        match (from, to) {
            // Created → Validated
            (TransactionState::Created, TransactionState::Validated) => true,

            // Validated → Signed | Rejected
            (TransactionState::Validated, TransactionState::Signed) => true,
            (TransactionState::Validated, TransactionState::Rejected) => true,

            // Signed → Broadcast
            (TransactionState::Signed, TransactionState::Broadcast) => true,

            // Broadcast → Confirmed | Rejected
            (TransactionState::Broadcast, TransactionState::Confirmed) => true,
            (TransactionState::Broadcast, TransactionState::Rejected) => true,

            // All others transitions are invalid
            _ => false,
        }
    }

    pub fn transition_to(
        &mut self,
        new_state: TransactionState,
    ) -> Result<(), StateTransitionError> {
        if !Self::can_transition(self.state, new_state) {
            return Err(StateTransitionError::InvalidTransition {
                from: self.state,
                to: new_state,
            });
        }
        self.state = new_state;
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
            InputKind::Coinbase { reward, .. } => *reward,
            InputKind::Regular { value, .. } => *value,
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
    let _ = (transaction, recipient);
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
            "{} sats to {} ({:?})",
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
            } => {
                write!(
                    _formatter,
                    "Regular input: {} sats from {} (sequence: {})",
                    value, previous_output, sequence
                )
            }
            InputKind::Coinbase {
                block_height,
                reward,
            } => {
                write!(
                    _formatter,
                    "Coinbase input: {} sats from block {} ",
                    reward, block_height
                )
            }
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(_formatter, "Transaction (v{})", self.version)?;
        writeln!(_formatter, "  Inputs ({}):", self.inputs.len())?;
        for input in &self.inputs {
            writeln!(_formatter, "    {}", input)?;
        }
        writeln!(_formatter, "  Outputs ({}):", self.outputs.len())?;
        for output in &self.outputs {
            writeln!(_formatter, "    {}", output)?;
        }
        writeln!(
            _formatter,
            "  Total in: {} sats, Total out: {} sats",
            self.total_input_value(),
            self.total_output_value()
        )?;
        match self.fee() {
            Ok(fee) => writeln!(_formatter, "  Fee: {} sats", fee)?,
            Err(_) => writeln!(_formatter, "  Fee: invalid")?,
        }
        writeln!(_formatter, "  Locktime: {}", self.locktime)
    }
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

impl fmt::Display for TransactionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Created => write!(f, "Created"),
            Self::Validated => write!(f, "Validated"),
            Self::Signed => write!(f, "Signed"),
            Self::Broadcast => write!(f, "Broadcast"),
            Self::Confirmed => write!(f, "Confirmed"),
            Self::Rejected => write!(f, "Rejected"),
        }
    }
}
