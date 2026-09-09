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
                write!(formatter, "EmptyTitle: The item has no title")
            }
            LibraryError::DuplicateItemId { id } => {
                write!(
                    formatter,
                    "DuplicateItemId: The item id: {id} already exist"
                )
            }
            LibraryError::DuplicateMemberId { id } => {
                write!(
                    formatter,
                    "DuplicateMemberId: The member id: {id} already exist"
                )
            }
            LibraryError::ItemNotFound { id } => {
                write!(formatter, "ItemNotFound: The item id: {id} does not exist")
            }
            LibraryError::MemberNotFound { id } => {
                write!(
                    formatter,
                    "MemberNotFound: The member id: {id} does not exist"
                )
            }
            LibraryError::ItemAlreadyOnLoan { id, member_id } => {
                write!(
                    formatter,
                    "ItemAlreadyOnLoan: The item id: {id} has being loaned to member id: {member_id}"
                )
            }
            LibraryError::ItemNotOnLoan { id } => {
                write!(formatter, "ItemNotOnLoan: The item id: {id} is not loaned")
            }
            LibraryError::ItemIsLost { id } => {
                write!(formatter, "ItemIsLost: The item id: {id} is lost")
            }
            LibraryError::BorrowLimitReached { member_id, limit } => {
                write!(
                    formatter,
                    "BorrowLimitReached: The member id : {member_id} has reached borrow limit of {limit}"
                )
            }
            LibraryError::InvalidReturnDay {
                day_borrowed,
                day_returned,
            } => {
                write!(
                    formatter,
                    "InvalidReturnDay: Item borrowed on {day_borrowed} was returned on {day_returned}"
                )
            }
        }
    }
}

impl std::error::Error for LibraryError {}
