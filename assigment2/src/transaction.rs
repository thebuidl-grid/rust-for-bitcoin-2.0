use std::collections::HashSet;
use std::fmt;

use crate::error::TransactionError;
use crate::utxo::UtxoSet;

/// Amount in satoshis (1 BTC = 100_000_000 sats).
pub type Sats = u64;

/// A simplified destination address (in real Bitcoin, a script or pubkey hash).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Address(pub String);

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Address {
    fn from(value: &str) -> Self {
        Address(value.to_string())
    }
}

/// A pointer to a specific output of a specific prior transaction.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OutPoint {
    pub txid: String,
    pub index: u32,
}

impl OutPoint {
    pub fn new(txid: impl Into<String>, index: u32) -> Self {
        OutPoint {
            txid: txid.into(),
            index,
        }
    }
}

impl fmt::Display for OutPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.txid, self.index)
    }
}

/// A newly created output: an amount locked to an address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxOutput {
    pub value: Sats,
    pub address: Address,
}

impl fmt::Display for TxOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} sats -> {}", self.value, self.address)
    }
}

/// Something that carries a satoshi amount. Implemented once and reused
/// anywhere a value needs to be summed (transaction outputs, UTXOs, ...)
/// instead of writing the same `.map(|x| x.value).sum()` in every module.
pub trait BitcoinValue {
    fn value_sats(&self) -> Sats;
}

impl BitcoinValue for TxOutput {
    fn value_sats(&self) -> Sats {
        self.value
    }
}

/// Sums the satoshi value of a collection of `BitcoinValue`s, checking for overflow.
pub fn total_value<'a, T: BitcoinValue + 'a>(
    items: impl IntoIterator<Item = &'a T>,
) -> Option<Sats> {
    items
        .into_iter()
        .try_fold(0u64, |acc, item| acc.checked_add(item.value_sats()))
}

/// A reference to a prior UTXO being spent, or the special coinbase input
/// that mints new coins as a block's mining reward (it spends nothing).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxInput {
    Regular { previous_output: OutPoint },
    Coinbase { block_height: u32 },
}

impl TxInput {
    pub fn previous_output(&self) -> Option<&OutPoint> {
        match self {
            TxInput::Regular { previous_output } => Some(previous_output),
            TxInput::Coinbase { .. } => None,
        }
    }
}

impl fmt::Display for TxInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TxInput::Regular { previous_output } => write!(f, "{previous_output}"),
            TxInput::Coinbase { block_height } => write!(f, "coinbase (block {block_height})"),
        }
    }
}

/// Lifecycle of a transaction as it moves from draft to confirmed on chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    Draft,
    Signed,
    Broadcast,
    Confirmed { height: u32 },
}

impl fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionStatus::Draft => write!(f, "draft"),
            TransactionStatus::Signed => write!(f, "signed"),
            TransactionStatus::Broadcast => write!(f, "broadcast"),
            TransactionStatus::Confirmed { height } => write!(f, "confirmed at height {height}"),
        }
    }
}

/// A validated transfer of value: inputs consumed, new outputs created.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub id: String,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u32,
    status: TransactionStatus,
}

/// Types that can check their own internal consistency.
pub trait Validate {
    fn validate(&self) -> Result<(), TransactionError>;
}

impl Transaction {
    /// Starts an empty draft transaction. Inputs and outputs are added one at
    /// a time with `add_input`/`add_output`; call `validate()` once the
    /// transaction is fully built.
    pub fn new(id: impl Into<String>, lock_time: u32) -> Self {
        Transaction {
            id: id.into(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            lock_time,
            status: TransactionStatus::Draft,
        }
    }

    /// Moves `input` into the transaction's input list. Takes `&mut self`
    /// because it mutates `self.inputs` in place rather than handing back a
    /// new `Transaction` each time.
    pub fn add_input(&mut self, input: TxInput) {
        self.inputs.push(input);
    }

    pub fn add_output(&mut self, output: TxOutput) {
        self.outputs.push(output);
    }

    pub fn status(&self) -> TransactionStatus {
        self.status
    }

    pub fn total_output_value(&self) -> Result<Sats, TransactionError> {
        total_value(&self.outputs).ok_or(TransactionError::AmountOverflow)
    }

    /// Sums the value of every input by looking each `Regular` input's
    /// previous output up in `utxo_set`; `Coinbase` inputs contribute 0,
    /// since they mint new coins rather than spending an existing one.
    /// Borrows both `self` and `utxo_set` immutably: it only needs to read
    /// them, so the caller keeps ownership of both afterward.
    pub fn total_input_value(&self, utxo_set: &UtxoSet) -> Result<Sats, TransactionError> {
        let mut total: Sats = 0;
        for input in &self.inputs {
            let value = match input.previous_output() {
                None => 0,
                Some(previous_output) => utxo_set
                    .get(previous_output)
                    .map(TxOutput::value_sats)
                    .ok_or_else(|| TransactionError::UtxoNotFound(previous_output.clone()))?,
            };
            total = total
                .checked_add(value)
                .ok_or(TransactionError::AmountOverflow)?;
        }
        Ok(total)
    }

    /// The transaction fee: total input value minus total output value.
    pub fn fee(&self, utxo_set: &UtxoSet) -> Result<Sats, TransactionError> {
        let input_total = self.total_input_value(utxo_set)?;
        let output_total = self.total_output_value()?;
        input_total
            .checked_sub(output_total)
            .ok_or(TransactionError::OutputsExceedInputs)
    }

    /// Advances the transaction to a new lifecycle state, rejecting any transition
    /// that doesn't follow draft -> signed -> broadcast -> confirmed.
    pub fn advance_status(&mut self, to: TransactionStatus) -> Result<(), TransactionError> {
        let allowed = matches!(
            (self.status, to),
            (TransactionStatus::Draft, TransactionStatus::Signed)
                | (TransactionStatus::Signed, TransactionStatus::Broadcast)
                | (
                    TransactionStatus::Broadcast,
                    TransactionStatus::Confirmed { .. }
                )
        );
        if allowed {
            self.status = to;
            Ok(())
        } else {
            Err(TransactionError::InvalidStateTransition {
                from: self.status,
                to,
            })
        }
    }
}

impl Validate for Transaction {
    fn validate(&self) -> Result<(), TransactionError> {
        if self.inputs.is_empty() {
            return Err(TransactionError::EmptyInputs);
        }
        if self.outputs.is_empty() {
            return Err(TransactionError::EmptyOutputs);
        }
        if self.outputs.iter().any(|output| output.value == 0) {
            return Err(TransactionError::ZeroValueOutput);
        }

        let mut seen = HashSet::with_capacity(self.inputs.len());
        for input in &self.inputs {
            if let Some(previous_output) = input.previous_output()
                && !seen.insert(previous_output)
            {
                return Err(TransactionError::DuplicateInput(previous_output.clone()));
            }
        }

        self.total_output_value()?;
        Ok(())
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Transaction {} [{}]", self.id, self.status)?;
        for input in &self.inputs {
            writeln!(f, "  in:  {input}")?;
        }
        for output in &self.outputs {
            writeln!(f, "  out: {output}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output(value: Sats) -> TxOutput {
        TxOutput {
            value,
            address: Address::from("addr1"),
        }
    }

    fn regular_input(txid: &str, index: u32) -> TxInput {
        TxInput::Regular {
            previous_output: OutPoint::new(txid, index),
        }
    }

    /// Builds a transaction from the given inputs/outputs without validating,
    /// mirroring how a caller assembles one via `add_input`/`add_output`.
    fn build(id: &str, inputs: Vec<TxInput>, outputs: Vec<TxOutput>) -> Transaction {
        let mut tx = Transaction::new(id, 0);
        for input in inputs {
            tx.add_input(input);
        }
        for output in outputs {
            tx.add_output(output);
        }
        tx
    }

    #[test]
    fn rejects_empty_inputs() {
        let tx = build("tx1", vec![], vec![output(100)]);
        assert_eq!(tx.validate().unwrap_err(), TransactionError::EmptyInputs);
    }

    #[test]
    fn rejects_empty_outputs() {
        let tx = build("tx1", vec![regular_input("prev", 0)], vec![]);
        assert_eq!(tx.validate().unwrap_err(), TransactionError::EmptyOutputs);
    }

    #[test]
    fn rejects_zero_value_output() {
        let tx = build("tx1", vec![regular_input("prev", 0)], vec![output(0)]);
        assert_eq!(
            tx.validate().unwrap_err(),
            TransactionError::ZeroValueOutput
        );
    }

    #[test]
    fn rejects_duplicate_inputs() {
        let tx = build(
            "tx1",
            vec![regular_input("prev", 0), regular_input("prev", 0)],
            vec![output(100)],
        );
        assert_eq!(
            tx.validate().unwrap_err(),
            TransactionError::DuplicateInput(OutPoint::new("prev", 0))
        );
    }

    #[test]
    fn coinbase_inputs_are_never_duplicates() {
        let tx = build(
            "tx1",
            vec![
                TxInput::Coinbase { block_height: 1 },
                TxInput::Coinbase { block_height: 1 },
            ],
            vec![output(100)],
        );
        assert!(tx.validate().is_ok());
    }

    #[test]
    fn accepts_valid_transaction() {
        let tx = build("tx1", vec![regular_input("prev", 0)], vec![output(100)]);
        tx.validate().unwrap();
        assert_eq!(tx.total_output_value().unwrap(), 100);
        assert_eq!(tx.status(), TransactionStatus::Draft);
    }

    #[test]
    fn total_output_value_detects_overflow() {
        let tx = build(
            "tx1",
            vec![regular_input("prev", 0)],
            vec![output(u64::MAX), output(1)],
        );
        assert_eq!(
            tx.total_output_value().unwrap_err(),
            TransactionError::AmountOverflow
        );
    }

    #[test]
    fn total_input_value_looks_up_the_utxo_set_and_treats_coinbase_as_free() {
        let mut utxo_set = UtxoSet::new();
        utxo_set.insert(OutPoint::new("prev", 0), output(500));

        let tx = build(
            "tx1",
            vec![
                regular_input("prev", 0),
                TxInput::Coinbase { block_height: 1 },
            ],
            vec![output(100)],
        );

        assert_eq!(tx.total_input_value(&utxo_set).unwrap(), 500);
        assert_eq!(tx.fee(&utxo_set).unwrap(), 400);
    }

    #[test]
    fn total_input_value_fails_when_the_utxo_is_missing() {
        let utxo_set = UtxoSet::new();
        let tx = build("tx1", vec![regular_input("prev", 0)], vec![output(100)]);
        assert_eq!(
            tx.total_input_value(&utxo_set).unwrap_err(),
            TransactionError::UtxoNotFound(OutPoint::new("prev", 0))
        );
    }

    #[test]
    fn status_transitions_follow_the_happy_path() {
        let mut tx = build("tx1", vec![regular_input("prev", 0)], vec![output(100)]);
        tx.advance_status(TransactionStatus::Signed).unwrap();
        tx.advance_status(TransactionStatus::Broadcast).unwrap();
        tx.advance_status(TransactionStatus::Confirmed { height: 800_000 })
            .unwrap();
        assert_eq!(
            tx.status(),
            TransactionStatus::Confirmed { height: 800_000 }
        );
    }

    #[test]
    fn status_rejects_skipping_a_state() {
        let mut tx = build("tx1", vec![regular_input("prev", 0)], vec![output(100)]);
        let err = tx.advance_status(TransactionStatus::Broadcast).unwrap_err();
        assert_eq!(
            err,
            TransactionError::InvalidStateTransition {
                from: TransactionStatus::Draft,
                to: TransactionStatus::Broadcast,
            }
        );
    }
}
