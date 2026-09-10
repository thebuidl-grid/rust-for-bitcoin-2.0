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
            LibraryError::EmptyTitle => write!(_formatter, "title cannot be empty"),
            LibraryError::DuplicateItemId { id } => {
                write!(_formatter, "item id {} already exists", id)
            }
            LibraryError::DuplicateMemberId { id } => {
                write!(_formatter, "member id {} already exists", id)
            }
            LibraryError::ItemNotFound { id } => {
                write!(_formatter, "item id {} not found", id)
            }
            LibraryError::MemberNotFound { id } => {
                write!(_formatter, "member id {} not found", id)
            }
            LibraryError::ItemAlreadyOnLoan { id, member_id } => {
                write!(
                    _formatter,
                    "item id {} is already on loan to member id {}",
                    id, member_id
                )
            }
            LibraryError::ItemNotOnLoan { id } => {
                write!(_formatter, "item id {} is not currently on loan", id)
            }
            LibraryError::ItemIsLost { id } => {
                write!(_formatter, "item id {} is marked as lost", id)
            }
            LibraryError::BorrowLimitReached { member_id, limit } => {
                write!(
                    _formatter,
                    "member id {} has reached the borrow limit of {}",
                    member_id, limit
                )
            }
            LibraryError::InvalidReturnDay {
                day_borrowed,
                day_returned,
            } => write!(
                _formatter,
                "invalid return day: borrowed on day {}, returned on day {}",
                day_borrowed, day_returned
            ),
        }
    }
}

impl std::error::Error for LibraryError {}
