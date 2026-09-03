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
        // TODO(Part 2): return a useful, human-readable message for every
        // variant. Include the ids and numbers the variant carries.
        match self {
            LibraryError::EmptyTitle => {
                write!(formatter, "Title cannot be empty")
            }
            LibraryError::DuplicateItemId { id } => {
                write!(
                    formatter,
                    "Item with ID {} already exists in the Library",
                    id
                )
            }
            LibraryError::DuplicateMemberId { id } => {
                write!(
                    formatter,
                    "A registered member with ID {} already exists",
                    id
                )
            }
            LibraryError::ItemNotFound { id } => {
                write!(formatter, "Item with ID {} not found", id)
            }
            LibraryError::MemberNotFound { id } => {
                write!(formatter, "Member with ID {} not found", id)
            }
            LibraryError::ItemAlreadyOnLoan { id, member_id } => {
                write!(
                    formatter,
                    "The requested item with ID {}; already on loan to member with ID {}",
                    id, member_id
                )
            }
            LibraryError::ItemNotOnLoan { id } => {
                write!(formatter, "Item with ID {} currently on loan", id)
            }
            LibraryError::ItemIsLost { id } => {
                write!(
                    formatter,
                    "Apologies!! item with ID {} has been marked lost and cannot be loaned",
                    id
                )
            }
            LibraryError::BorrowLimitReached { member_id, limit } => {
                write!(
                    formatter,
                    "Member {} has reached borrow limit of {}",
                    member_id, limit
                )
            }
            LibraryError::InvalidReturnDay {
                day_borrowed,
                day_returned,
            } => {
                write!(
                    formatter,
                    "Invalid return day: item borrowed on {} but return attempted on day {}",
                    day_borrowed, day_returned
                )
            }
        }
    }
}

impl std::error::Error for LibraryError {}
