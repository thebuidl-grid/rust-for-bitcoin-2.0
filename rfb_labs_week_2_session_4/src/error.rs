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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LibraryError::EmptyTitle => write!(f, "Item title cannot be empty"),
            LibraryError::DuplicateItemId { id } => write!(f, "Item with id {id} already exists"),
            LibraryError::DuplicateMemberId { id } => {
                write!(f, "Member with id {id} already registered")
            }
            LibraryError::ItemNotFound { id } => write!(f, "Item with id {id} not found"),
            LibraryError::MemberNotFound { id } => write!(f, "Member with id {id} not found"),
            LibraryError::ItemAlreadyOnLoan { id, member_id } => write!(
                f,
                "Item with id {id} is already on loan to member {member_id}"
            ),
            LibraryError::ItemNotOnLoan { id } => write!(f, "Item with id {id} is not on loan"),
            LibraryError::ItemIsLost { id } => write!(f, "Item with id {id} is lost"),
            LibraryError::BorrowLimitReached { member_id, limit } => write!(
                f,
                "Member {member_id} reached the borrow limit of {limit} items"
            ),
            LibraryError::InvalidReturnDay {
                day_borrowed,
                day_returned,
            } => write!(
                f,
                "Return day {day_returned} cannot be earlier than borrow day {day_borrowed}"
            ),
        }
    }
}

impl std::error::Error for LibraryError {}
