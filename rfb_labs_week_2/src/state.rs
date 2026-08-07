//! Part 10 (optional): the lifecycle a transaction moves through, modelled so
//! that invalid transitions are rejected rather than silently accepted.

use std::fmt;

/// Stages a transaction passes through, from construction to its final outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    Created,
    Validated,
    Signed,
    Broadcast,
    Confirmed,
    Rejected,
}

/// Returned when a caller asks for a transition the lifecycle does not allow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTransition {
    pub from: TransactionState,
    pub to: TransactionState,
}

impl TransactionState {
    /// `Confirmed` and `Rejected` are terminal: nothing follows them.
    pub fn is_final(self) -> bool {
        matches!(
            self,
            TransactionState::Confirmed | TransactionState::Rejected
        )
    }

    /// The happy path runs Created → Validated → Signed → Broadcast → Confirmed.
    /// Any non-terminal state may instead fail into `Rejected`.
    pub fn can_transition_to(self, next: TransactionState) -> bool {
        let advances = matches!(
            (self, next),
            (TransactionState::Created, TransactionState::Validated)
                | (TransactionState::Validated, TransactionState::Signed)
                | (TransactionState::Signed, TransactionState::Broadcast)
                | (TransactionState::Broadcast, TransactionState::Confirmed)
        );

        advances || (next == TransactionState::Rejected && !self.is_final())
    }
}

/// Owns a transaction's current state and is the only way to change it, so an
/// illegal jump such as Created → Broadcast cannot be expressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionLifecycle {
    state: TransactionState,
}

impl TransactionLifecycle {
    pub fn new() -> Self {
        Self {
            state: TransactionState::Created,
        }
    }

    pub fn state(&self) -> TransactionState {
        self.state
    }

    /// Advances to `next`, or leaves the state untouched and reports why not.
    pub fn advance_to(
        &mut self,
        next: TransactionState,
    ) -> Result<TransactionState, InvalidTransition> {
        if !self.state.can_transition_to(next) {
            return Err(InvalidTransition {
                from: self.state,
                to: next,
            });
        }

        self.state = next;
        Ok(self.state)
    }
}

impl Default for TransactionLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TransactionState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            TransactionState::Created => "created",
            TransactionState::Validated => "validated",
            TransactionState::Signed => "signed",
            TransactionState::Broadcast => "broadcast",
            TransactionState::Confirmed => "confirmed",
            TransactionState::Rejected => "rejected",
        };

        formatter.write_str(label)
    }
}

impl fmt::Display for InvalidTransition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "a {} transaction cannot move to {}",
            self.from, self.to
        )
    }
}

impl std::error::Error for InvalidTransition {}
