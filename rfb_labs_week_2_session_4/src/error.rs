use std::fmt;

/// ordinary outcomes a caller is expected to handle.

#[derive(Debug, PartialEq, Eq)]
pub enum LibraryError {
    EmptyTitle,
    DuplicateItemId {
        id: u32,
    },
    DuplicateMemberId {
        id: u32,
    },
    ItemNotFound {
        id: u32,
    },
    MemberNotFound {
        id: u32,
    },
    ItemAlreadyOnLoan {
        id: u32,
        member_id: u32,
    },
    ItemNotOnLoan {
        id: u32,
    },
    ItemIsLost {
        id: u32,
    },
    BorrowLimitReached {
        member_id: u32,
        limit: usize,
    },
    InvalidReturnDay {
        day_borrowed: u32,
        day_returned: u32,
    },
}

impl fmt::Display for LibraryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LibraryError::EmptyTitle => write!(formatter, "item title cannot be empty"),
            LibraryError::DuplicateItemId { id } => {
                write!(formatter, "item with id {id} already exists")
            }
            LibraryError::DuplicateMemberId { id } => {
                write!(formatter, "member with id {id} is already registered")
            }
            LibraryError::ItemNotFound { id } => {
                write!(formatter, "item with id {id} not found")
            }
            LibraryError::MemberNotFound { id } => {
                write!(formatter, "member with id {id} not found")
            }
            LibraryError::ItemAlreadyOnLoan { id, member_id } => {
                write!(
                    formatter,
                    "item {id} is already on loan to member {member_id}"
                )
            }
            LibraryError::ItemNotOnLoan { id } => {
                write!(formatter, "item {id} is not currently on loan")
            }
            LibraryError::ItemIsLost { id } => {
                write!(formatter, "item {id} is marked as lost")
            }
            LibraryError::BorrowLimitReached { member_id, limit } => {
                write!(
                    formatter,
                    "member {member_id} reached borrow limit of {limit} items"
                )
            }
            LibraryError::InvalidReturnDay {
                day_borrowed,
                day_returned,
            } => {
                write!(
                    formatter,
                    "invalid return day {day_returned}: borrowed on day {day_borrowed}"
                )
            }
        }
    }
}

impl std::error::Error for LibraryError {}