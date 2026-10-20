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
            LibraryError::EmptyTitle => {
                write!(formatter, "item title cannot be empty")
            }
            LibraryError::DuplicateItemId { id } => {
                write!(formatter, "item with id {} already exists", id)
            }
            LibraryError::DuplicateMemberId { id } => {
                write!(formatter, "member with id {} already exists", id)
            }
            LibraryError::ItemNotFound { id } => {
                write!(formatter, "item with id {} was not found", id)
            }
            LibraryError::MemberNotFound { id } => {
                write!(formatter, "member with id {} was not found", id)
            }
            LibraryError::ItemAlreadyOnLoan { id, member_id } => {
                write!(
                    formatter,
                    "item {} is already on loan to member {}",
                    id, member_id
                )
            }
            LibraryError::ItemNotOnLoan { id } => {
                write!(formatter, "item {} is not currently on loan", id)
            }
            LibraryError::ItemIsLost { id } => {
                write!(formatter, "item {} is lost", id)
            }
            LibraryError::BorrowLimitReached { member_id, limit } => {
                write!(
                    formatter,
                    "member {} has reached the borrow limit of {} items",
                    member_id, limit
                )
            }
            LibraryError::InvalidReturnDay {
                day_borrowed,
                day_returned,
            } => {
                write!(
                    formatter,
                    "return day {} is before borrow day {}",
                    day_returned, day_borrowed
                )
            }
        }
    }
}

impl std::error::Error for LibraryError {}
