use crate::error::TransactionError;
use crate::transaction::Transaction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TxState {
    Created,
    Validated,
    Signed,
    Broadcast,
    Confirmed,
    Rejected,
}

pub struct StatefulTransaction {
    pub transaction: Transaction,
    pub state: TxState,
}

impl StatefulTransaction {
    pub fn new(transaction: Transaction) -> Self {
        Self {
            transaction,
            state: TxState::Created,
        }
    }

    pub fn transition_to(&mut self, next: TxState) -> Result<(), TransactionError> {
        use TxState::*;

        let allowed = matches!(
            (self.state, next),
            (Created, Validated)
                | (Validated, Signed)
                | (Validated, Rejected)
                | (Signed, Broadcast)
                | (Broadcast, Confirmed)
                | (Broadcast, Rejected)
        );

        if allowed {
            self.state = next;
            Ok(())
        } else {
            Err(TransactionError::InvalidStateTransition {
                from: self.state,
                to: next,
            })
        }
    }
}
