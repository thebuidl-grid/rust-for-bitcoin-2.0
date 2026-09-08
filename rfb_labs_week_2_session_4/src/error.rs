use std::fmt::{self, write};

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
        // TODO(Part 2): return a useful, human-readable message for every
        // variant. Include the ids and numbers the variant carries.
        // todo!("implement Display for LibraryError")
        match self {
            LibraryError::EmptyTitle => write!(_formatter, "empty title"),
            LibraryError::DuplicateItemId { id } => write!(_formatter, "duplicate item, the id is {id}"),
            LibraryError::DuplicateMemberId { id } => write!(_formatter, "duplicate member, the id is {id}"),
            LibraryError::ItemNotFound { id } => write!(_formatter, "item {id} not found"),
            LibraryError::MemberNotFound { id } => write!(_formatter, "member {id} not found"),
            LibraryError::ItemAlreadyOnLoan { id, member_id } => write!(_formatter, "item {id} already on loan by member {member_id}"),
            LibraryError::ItemNotOnLoan { id } => write!(_formatter, "item {id} not on loan"),
            LibraryError::ItemIsLost { id } => write!(_formatter, "item {id} is lost"),
            LibraryError::BorrowLimitReached { member_id, limit } => write!(_formatter, "member {member_id} has reached borrow limit. Limtt {limit}"),
            LibraryError::InvalidReturnDay { .. } => write!(_formatter, "invalid return day")
        }
    }
}

impl std::error::Error for LibraryError {}
