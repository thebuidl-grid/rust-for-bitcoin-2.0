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
        // TODO(Part 2): return a useful, human-readable message for every
        // variant. Include the ids and numbers the variant carries.
        // todo!("implement Display for LibraryError")
        match self {
            LibraryError::EmptyTitle => write!(_formatter, "Empty title"),
            LibraryError::DuplicateItemId { id } => write!(_formatter, "Duplicate item Id {}", id),
            LibraryError::DuplicateMemberId { id } => {
                write!(_formatter, "Duplicate member Id {}", id)
            }
            LibraryError::ItemNotFound { id } => {
                write!(_formatter, "Item with id {} not found ", id)
            }
            LibraryError::MemberNotFound { id } => {
                write!(_formatter, "Member with id {} not found ", id)
            }
            LibraryError::ItemAlreadyOnLoan { id, member_id } => write!(
                _formatter,
                "Item with id {} already on loan to member with id - {} ",
                id, member_id
            ),
            LibraryError::ItemNotOnLoan { id } => {
                write!(_formatter, "Item with id {} not on loan to any member", id)
            }
            LibraryError::ItemIsLost { id } => write!(_formatter, "Item with id {} is lost", id),
            LibraryError::BorrowLimitReached { member_id, limit } => write!(
                _formatter,
                "Borrow limit {} of {} is reached",
                limit, member_id
            ),
            LibraryError::InvalidReturnDay {
                day_borrowed,
                day_returned,
            } => write!(
                _formatter,
                "Item was borrowed on {} but returned on {}",
                day_borrowed, day_returned
            ),
        }
    }
}

impl std::error::Error for LibraryError {}
