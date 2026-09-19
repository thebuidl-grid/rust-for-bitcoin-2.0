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
        self.inputs.iter().map(|input| input.value()).sum()
    }

    pub fn total_output_value(&self) -> u64 {
        self.outputs.iter().map(|output| output.value()).sum()
    }

    pub fn fee(&self) -> Result<u64, TransactionError> {
        let total_in = self.total_input_value();
        let total_out = self.total_output_value();
        total_in
            .checked_sub(total_out)
            .ok_or(TransactionError::OutputsExceedInputs {
                total_inputs: total_in,
                total_outputs: total_out,
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

        let mut coinbase_count = 0;
        let mut regular_count = 0;

        for input in &self.inputs {
            match input {
                InputKind::Regular {
                    previous_output, ..
                } => {
                    regular_count += 1;
                    if previous_output.txid.trim().is_empty() {
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
    transaction.outputs.iter().max_by_key(|o| o.value)
}

pub fn find_outputs_for_recipient<'a>(
    transaction: &'a Transaction,
    recipient: &str,
) -> Vec<&'a TxOutput> {
    transaction
        .outputs
        .iter()
        .filter(|o| o.recipient == recipient)
        .collect()
}

impl fmt::Display for OutPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.txid, self.vout)
    }
}

impl fmt::Display for TxOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} sats ({:.8} BTC) -> {} [{:?}]",
            self.value,
            self.value_in_btc(),
            self.recipient,
            self.output_type
        )
    }
}

impl fmt::Display for InputKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputKind::Regular {
                previous_output,
                value,
                sequence,
            } => {
                write!(
                    f,
                    "Regular Input [{}] ({} sats / {:.8} BTC, sequence: {})",
                    previous_output,
                    *value,
                    *value as f64 / 100_000_000.0,
                    sequence
                )
            }
            InputKind::Coinbase {
                block_height,
                reward,
            } => {
                write!(
                    f,
                    "Coinbase Input [block #{}] (reward: {} sats / {:.8} BTC)",
                    block_height,
                    *reward,
                    *reward as f64 / 100_000_000.0
                )
            }
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== Bitcoin Transaction Summary ===")?;
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Locktime: {}", self.locktime)?;
        writeln!(f, "Inputs ({}):", self.inputs.len())?;
        for (i, input) in self.inputs.iter().enumerate() {
            writeln!(f, "  [{i}] {input}")?;
        }
        writeln!(f, "Outputs ({}):", self.outputs.len())?;
        for (i, output) in self.outputs.iter().enumerate() {
            writeln!(f, "  [{i}] {output}")?;
        }
        let total_in = self.total_input_value();
        let total_out = self.total_output_value();
        writeln!(
            f,
            "Total Input Value:  {} sats ({:.8} BTC)",
            total_in,
            total_in as f64 / 100_000_000.0
        )?;
        writeln!(
            f,
            "Total Output Value: {} sats ({:.8} BTC)",
            total_out,
            total_out as f64 / 100_000_000.0
        )?;
        match self.fee() {
            Ok(fee_val) => writeln!(
                f,
                "Calculated Fee:     {} sats ({:.8} BTC)",
                fee_val,
                fee_val as f64 / 100_000_000.0
            ),
            Err(err) => writeln!(f, "Calculated Fee:     Invalid Fee ({err})"),
        }
    }
}
