use crate::error::TransactionError;
use crate::transaction::Transaction;
use std::fmt;

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
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            TransactionState::Created => "Created",
            TransactionState::Validated => "Validated",
            TransactionState::Signed => "Signed",
            TransactionState::Broadcast => "Broadcasted",
            TransactionState::Confirmed => "Confirmed",
            TransactionState::Rejected => "Rejected",
        };
        write!(formatter, "{label}")
    }
}

pub struct TrackedTransaction {
    pub transaction: Transaction,
    state: TransactionState,
}

impl TrackedTransaction {
    pub fn new(transaction: Transaction) -> Self {
        Self {
            transaction,
            state: TransactionState::Created,
        }
    }

    pub fn state(&self) -> TransactionState {
        self.state
    }

    fn transition(&mut self, to: TransactionState) -> Result<(), TransactionError> {
        let allowed = matches!(
            (self.state, to),
            (TransactionState::Created, TransactionState::Validated)
                | (TransactionState::Created, TransactionState::Rejected)
                | (TransactionState::Validated, TransactionState::Signed)
                | (TransactionState::Validated, TransactionState::Rejected)
                | (TransactionState::Signed, TransactionState::Broadcast)
                | (TransactionState::Signed, TransactionState::Rejected)
                | (TransactionState::Broadcast, TransactionState::Confirmed)
                | (TransactionState::Broadcast, TransactionState::Rejected)
        );

        if !allowed {
            return Err(TransactionError::InvalidStateTransition {
                from: self.state,
                to,
            });
        }
        self.state = to;
        Ok(())
    }

    pub fn mark_validated(&mut self) -> Result<(), TransactionError> {
        self.transaction.validate()?;
        self.transition(TransactionState::Validated)
    }

    pub fn mark_signed(&mut self) -> Result<(), TransactionError> {
        self.transition(TransactionState::Signed)
    }

    pub fn mark_broadcast(&mut self) -> Result<(), TransactionError> {
        self.transition(TransactionState::Broadcast)
    }

    pub fn mark_confirmed(&mut self) -> Result<(), TransactionError> {
        self.transition(TransactionState::Confirmed)
    }

    pub fn mark_rejected(&mut self) -> Result<(), TransactionError> {
        self.transition(TransactionState::Rejected)
    }
}
