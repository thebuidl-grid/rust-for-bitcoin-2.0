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
        let name = match self {
            TransactionState::Created => "created",
            TransactionState::Validated => "validated",
            TransactionState::Signed => "signed",
            TransactionState::Broadcast => "broadcast",
            TransactionState::Confirmed => "confirmed",
            TransactionState::Rejected => "rejected",
        };
        write!(formatter, "{name}")
    }
}
