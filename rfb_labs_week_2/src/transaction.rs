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
        let total_input = self.total_input_value();
        let total_output = self.total_output_value();

        total_input
            .checked_sub(total_output)
            .ok_or(TransactionError::OutputsExceedInputs {
                total_inputs: total_input,
                total_outputs: total_output,
            })
    }

    pub fn validate(&self) -> Result<(), TransactionError> {
        // No inputs
        if self.inputs.is_empty() {
            return Err(TransactionError::NoInputs);
        }
        // No outputs
        if self.outputs.is_empty() {
            return Err(TransactionError::NoOutputs);
        }
        // Non-OpReturn zero-value outputs
        for output in &self.outputs {
            if output.value == 0 && output.output_type != OutputType::OpReturn {
                return Err(TransactionError::ZeroValueOutput);
            }
        }
        //Count Coinbase inputs and check for mixing -> Coinbase Rule
        let coinbase_count = self
            .inputs
            .iter()
            .filter(|input| matches!(input, InputKind::Coinbase { .. }))
            .count();
        if coinbase_count > 1 {
            return Err(TransactionError::MultipleCoinbaseInputs);
        }

        if coinbase_count > 0 && coinbase_count < self.inputs.len() {
            return Err(TransactionError::CoinbaseMixedWithRegularInputs);
        }
        //Regular-input txids is empty
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
        // Outputs exceeding inputs
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
            "{} sats -> {} ({:?})",
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
                "Regular input {} ({} sats, sequence {})",
                previous_output, value, sequence
            ),

            InputKind::Coinbase {
                block_height,
                reward,
            } => write!(
                _formatter,
                "Coinbase input at block_height {}  ({} sats)",
                block_height, reward
            ),
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fee_display = match self.fee() {
            Ok(fee) => fee.to_string(),
            Err(err) => format!("Invalid ({err})"),
        };

        write!(
            _formatter,
            "Transaction version {}
            (locktime {})\n
            inputs: {}
            (total {} sats)\n
            outputs: {}
            (total {} sats)\n
            fee: {}
            ",
            self.version,
            self.locktime,
            self.inputs.len(),
            self.total_input_value(),
            self.outputs.len(),
            self.total_output_value(),
            fee_display
        )
    }
}
