use crate::transaction::Transaction;

/// The lifecycle stages for the optional transaction-state exercise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    Created,
    Validated,
    Signed,
    Broadcast,
    Confirmed,
    Rejected,
}

/// Owns a transaction and permits only valid state transitions.
#[derive(Debug, PartialEq, Eq)]
pub struct TransactionLifecycle {
    transaction: Transaction,
    state: TransactionState,
}

impl TransactionLifecycle {
    pub fn new(transaction: Transaction) -> Self {
        Self {
            transaction,
            state: TransactionState::Created,
        }
    }

    pub fn state(&self) -> TransactionState {
        self.state
    }

    pub fn transaction(&self) -> &Transaction {
        &self.transaction
    }

    /// A well-formed transaction becomes `Validated`; an invalid one becomes `Rejected`.
    pub fn validate(mut self) -> Result<Self, Self> {
        if self.state != TransactionState::Created {
            return Err(self);
        }

        self.state = if self.transaction.validate().is_ok() {
            TransactionState::Validated
        } else {
            TransactionState::Rejected
        };
        Ok(self)
    }

    pub fn sign(mut self) -> Result<Self, Self> {
        if self.state != TransactionState::Validated {
            return Err(self);
        }

        self.state = TransactionState::Signed;
        Ok(self)
    }

    pub fn broadcast(mut self) -> Result<Self, Self> {
        if self.state != TransactionState::Signed {
            return Err(self);
        }

        self.state = TransactionState::Broadcast;
        Ok(self)
    }

    pub fn confirm(mut self) -> Result<Self, Self> {
        if self.state != TransactionState::Broadcast {
            return Err(self);
        }

        self.state = TransactionState::Confirmed;
        Ok(self)
    }

    pub fn reject(mut self) -> Result<Self, Self> {
        if matches!(
            self.state,
            TransactionState::Confirmed | TransactionState::Rejected
        ) {
            return Err(self);
        }

        self.state = TransactionState::Rejected;
        Ok(self)
    }
}
