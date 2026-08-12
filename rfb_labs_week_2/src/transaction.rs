use std::fmt;

use crate::error::TransactionError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputType {
    P2pkh,
    P2wpkh,
    P2tr,
    OpReturn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxOutput {
    pub value: u64,
    pub recipient: String,
    pub output_type: OutputType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutPoint {
    pub txid: String,
    pub vout: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
        // let _ = input;
        // todo!("add an input")

        self.inputs.push(input);
    }

    pub fn add_output(&mut self, output: TxOutput) {
        // TODO(Part 3): move `output` into the transaction.
        // let _ = output;
        // todo!("add an output")

        self.outputs.push(output);
    }

    pub fn total_input_value(&self) -> u64 {
        // TODO(Part 3): match both InputKind variants and sum their values.
        // todo!("calculate the total input value")
        self.inputs.iter().map(|input| input.value()).sum()
    }

    pub fn total_output_value(&self) -> u64 {
        // TODO(Part 3): sum the value of every output.
        // todo!("calculate the total output value")
        self.outputs.iter().map(|output| output.value()).sum()
    }

    pub fn fee(&self) -> Result<u64, TransactionError> {
        // / TODO(Part 3): checked subtraction must return OutputsExceedInputs.
        // todo!("calculate the fee")
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
        // TODO(Part 5): apply every validation rule in ASSIGNMENT.md.
        // todo!("validate the transaction")

        // Reject empty inputs
        if self.inputs.is_empty() {
            return Err(TransactionError::NoInputs);
        }

        // Reject empty outputs
        if self.outputs.is_empty() {
            return Err(TransactionError::NoOutputs);
        }

        // Reject non-OpReturn outputs with 0 value
        for output in &self.outputs {
            if output.value == 0 && output.output_type != OutputType::OpReturn {
                return Err(TransactionError::ZeroValueOutput);
            }
        }

        // Input classification checks (Coinbase vs Regular)
        let mut coinbase_count = 0;
        let mut regular_count = 0;

        for input in &self.inputs {
            match input {
                InputKind::Coinbase { .. } => {
                    coinbase_count += 1;
                }
                InputKind::Regular {
                    previous_output, ..
                } => {
                    regular_count += 1;
                    // Reject empty or whitespace-only TXIDs
                    if previous_output.txid.trim().is_empty() {
                        return Err(TransactionError::InvalidTxid);
                    }
                }
            }
        }

        // Reject mixing coinbase and regular inputs
        if coinbase_count > 0 && regular_count > 0 {
            return Err(TransactionError::CoinbaseMixedWithRegularInputs);
        }

        // Reject multiple coinbase inputs
        if coinbase_count > 1 {
            return Err(TransactionError::MultipleCoinbaseInputs);
        }

        // 5. Ensure total_inputs >= total_outputs
        // Calling self.fee()? automatically checks total_input_value().checked_sub(...)
        // and returns Err(TransactionError::OutputsExceedInputs) if outputs exceed inputs.
        let _fee = self.fee()?;

        Ok(())
    }
}

// tx.validate()?;
//     match tx.validate() {
//     Ok(()) => println!("Transaction is valid!"),
//     Err(e) => println!("Validation failed: {}", e),
// }

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

        // Part 6: Pattern match to extract the value regardless of the variant.
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

    // Part 7: Iterators combined with max_by_key easily solve this
    // while perfectly preserving the borrowed lifetime.
    transaction.outputs.iter().max_by_key(|out| out.value)

    // transaction.outputs.into_iter().max_by_key(|out| out.value)
}

pub fn find_outputs_for_recipient<'a>(
    transaction: &'a Transaction,
    recipient: &str,
) -> Vec<&'a TxOutput> {
    // TODO(Part 7): return references to all matching outputs.
    // let _ = (transaction, reci.pient);
    // todo!("find outputs for a recipient")

    // Part 7: filter collects matching references into a new vector.
    // The explicit 'a lifetime ensures the returned references live
    // exactly as long as the transaction reference.
    transaction
        .outputs
        .iter()
        .filter(|out| out.recipient == recipient)
        .collect()
}

pub mod state {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Created;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Validated;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Signed {
        pub signature: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Broadcast {
        pub txid: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Confirmed {
        pub block_height: u32,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Rejected {
        pub reason: String,
    }
}

/// A state-aware wrapper around a `Transaction`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxState<S> {
    pub inner: Transaction,
    pub state_data: S,
}

impl TxState<state::Created> {
    /// Constructs a new state machine in the initial `Created` state.
    pub fn new(tx: Transaction) -> Self {
        Self {
            inner: tx,
            state_data: state::Created,
        }
    }

    /// Transitions from `Created` -> `Validated` (or `Rejected` if invalid).
    pub fn validate(self) -> Result<TxState<state::Validated>, TxState<state::Rejected>> {
        match self.inner.validate() {
            Ok(()) => Ok(TxState {
                inner: self.inner,
                state_data: state::Validated,
            }),
            Err(err) => Err(TxState {
                inner: self.inner,
                state_data: state::Rejected {
                    reason: err.to_string(),
                },
            }),
        }
    }
}

impl TxState<state::Validated> {
    /// Transitions from `Validated` -> `Signed`.
    pub fn sign(self, signature: impl Into<String>) -> TxState<state::Signed> {
        TxState {
            inner: self.inner,
            state_data: state::Signed {
                signature: signature.into(),
            },
        }
    }
}

impl TxState<state::Signed> {
    /// Transitions from `Signed` -> `Broadcast`.
    pub fn broadcast(self, txid: impl Into<String>) -> TxState<state::Broadcast> {
        TxState {
            inner: self.inner,
            state_data: state::Broadcast { txid: txid.into() },
        }
    }
}

impl TxState<state::Broadcast> {
    /// Transitions from `Broadcast` -> `Confirmed`.
    pub fn confirm(self, block_height: u32) -> TxState<state::Confirmed> {
        TxState {
            inner: self.inner,
            state_data: state::Confirmed { block_height },
        }
    }
}

// Any state can transition into `Rejected`
impl<S> TxState<S> {
    pub fn reject(self, reason: impl Into<String>) -> TxState<state::Rejected> {
        TxState {
            inner: self.inner,
            state_data: state::Rejected {
                reason: reason.into(),
            },
        }
    }
}

impl fmt::Display for OutPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Formats as `<txid>:<vout>`
        write!(f, "{}:{}", self.txid, self.vout)
    }
}

impl fmt::Display for TxOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Output(Value: {} sats, Recipient: {}, Type: {:?})",
            self.value, self.recipient, self.output_type
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
                // Uses OutPoint's Display trait automatically via `{}`
                write!(
                    f,
                    "Input(OutPoint: {}, Value: {} sats, Seq: {})",
                    previous_output, value, sequence
                )
            }
            InputKind::Coinbase {
                block_height,
                reward,
            } => {
                write!(
                    f,
                    "Coinbase(Height: {}, Reward: {} sats)",
                    block_height, reward
                )
            }
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Transaction (Version: {}, Locktime: {})",
            self.version, self.locktime
        )?;

        writeln!(f, "Inputs ({}):", self.inputs.len())?;
        if self.inputs.is_empty() {
            writeln!(f, "  (none)")?;
        } else {
            for (i, input) in self.inputs.iter().enumerate() {
                // Calls InputKind's Display implementation automatically via {}
                writeln!(f, "  [{}] {}", i, input)?;
            }
        }

        writeln!(f, "Outputs ({}):", self.outputs.len())?;
        if self.outputs.is_empty() {
            writeln!(f, "  (none)")?;
        } else {
            for (i, output) in self.outputs.iter().enumerate() {
                // Calls TxOutput's Display implementation automatically via {}
                writeln!(f, "  [{}] {}", i, output)?;
            }
        }

        let total_in = self.total_input_value();
        let total_out = self.total_output_value();
        writeln!(
            f,
            "Total In: {} sats | Total Out: {} sats",
            total_in, total_out
        )?;

        // Safely check and print the fee using the fee() method
        match self.fee() {
            Ok(fee) => write!(f, "Fee: {} sats", fee),
            Err(_) => write!(f, "Fee: INVALID (outputs exceed inputs)"),
        }
    }
}
