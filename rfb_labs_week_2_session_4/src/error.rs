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
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTitle => write!(formatter, "an item needs a title"),
            Self::DuplicateItemId { id } => {
                write!(formatter, "item {id} is already stocked")
            }
            Self::DuplicateMemberId { id } => {
                write!(formatter, "member {id} is already registered")
            }
            Self::ItemNotFound { id } => write!(formatter, "no item with id {id}"),
            Self::MemberNotFound { id } => write!(formatter, "no member with id {id}"),
            Self::ItemAlreadyOnLoan { id, member_id } => write!(
                formatter,
                "item {id} is already on loan to member {member_id}"
            ),
            Self::ItemNotOnLoan { id } => {
                write!(
                    formatter,
                    "item {id} is not on loan, so it cannot be returned"
                )
            }
            Self::ItemIsLost { id } => write!(formatter, "item {id} is lost"),
            Self::BorrowLimitReached { member_id, limit } => write!(
                formatter,
                "member {member_id} already holds the limit of {limit} items"
            ),
            Self::InvalidReturnDay {
                day_borrowed,
                day_returned,
            } => write!(
                formatter,
                "day {day_returned} is before the borrow day {day_borrowed}"
            ),
        }
    }
}

impl std::error::Error for LibraryError {}
