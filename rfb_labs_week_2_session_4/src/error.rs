use std::fmt;

/// Every expected failure in the lending library.
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
            Self::EmptyTitle => write!(formatter, "item title cannot be empty"),
            Self::DuplicateItemId { id } => {
                write!(formatter, "an item with id {id} already exists")
            }
            Self::DuplicateMemberId { id } => {
                write!(formatter, "a member with id {id} is already registered")
            }
            Self::ItemNotFound { id } => write!(formatter, "item {id} was not found"),
            Self::MemberNotFound { id } => write!(formatter, "member {id} was not found"),
            Self::ItemAlreadyOnLoan { id, member_id } => {
                write!(
                    formatter,
                    "item {id} is already on loan to member {member_id}"
                )
            }
            Self::ItemNotOnLoan { id } => {
                write!(formatter, "item {id} is not currently on loan")
            }
            Self::ItemIsLost { id } => write!(formatter, "item {id} is lost"),
            Self::BorrowLimitReached { member_id, limit } => {
                write!(
                    formatter,
                    "member {member_id} has reached the borrow limit of {limit}"
                )
            }
            Self::InvalidReturnDay {
                day_borrowed,
                day_returned,
            } => write!(
                formatter,
                "return day {day_returned} is earlier than borrow day {day_borrowed}"
            ),
        }
    }
}

impl std::error::Error for LibraryError {}
