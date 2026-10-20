use std::fmt;

/// Every expected failure in the lending library.
///
/// This is the only file whose types are written for you. Nothing here should
/// ever be produced by a `panic!`, an `unwrap`, or an `expect` — these are
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
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTitle => write!(_formatter, "item has an empty title"),
            Self::DuplicateItemId { id } => write!(_formatter, "id: {id} already exists"),
            Self::DuplicateMemberId { id } => {
                write!(_formatter, "member with id: {id} already exists")
            }
            Self::ItemNotFound { id } => write!(_formatter, "item with id: {id} not found"),
            Self::MemberNotFound { id } => write!(_formatter, "member with id: {id} not found"),
            Self::ItemAlreadyOnLoan { id, member_id } => write!(
                _formatter,
                "item with id: {id} is already on loan to member with id: {member_id}"
            ),
            Self::ItemNotOnLoan { id } => write!(_formatter, "item with id: {id} is not on loan"),
            Self::ItemIsLost { id } => write!(_formatter, "item with id: {id} is lost"),
            Self::BorrowLimitReached { member_id, limit } => write!(
                _formatter,
                "member with id: {member_id} has reached borrow limit of {limit}"
            ),
            Self::InvalidReturnDay {
                day_borrowed,
                day_returned,
            } => write!(
                _formatter,
                "day returned {day_returned} is before the day it was borrowed {day_borrowed}"
            ),
        }
    }
}

impl std::error::Error for LibraryError {}
