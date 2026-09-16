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
        // todo!("implement Display for LibraryError")
        match self {
            Self::EmptyTitle => write!(formatter, "item title cannot be empty"),
            Self::DuplicateItemId { id } => write!(formatter, "item {} already exists", id),
            Self::DuplicateMemberId { id } => write!(formatter, "member {} already registered", id),
            Self::ItemNotFound { id } => write!(formatter, "item {} not found", id),
            Self::MemberNotFound { id } => write!(formatter, "member {} not found", id),
            Self::ItemAlreadyOnLoan { id, member_id } => {
                write!(
                    formatter,
                    "item {} already checked out to member {}",
                    id, member_id
                )
            }
            Self::ItemNotOnLoan { id } => write!(formatter, "item {} is not on loan", id),
            Self::ItemIsLost { id } => write!(formatter, "item {} is lost", id),
            Self::BorrowLimitReached { member_id, limit } => {
                write!(
                    formatter,
                    "member {} has reached the borrow limit of {}",
                    member_id, limit
                )
            }
            Self::InvalidReturnDay {
                day_borrowed,
                day_returned,
            } => {
                write!(
                    formatter,
                    "cannot return on day {} (borrowed on day {})",
                    day_returned, day_borrowed
                )
            }
        }
    }
}

impl std::error::Error for LibraryError {}
