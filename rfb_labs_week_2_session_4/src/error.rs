use std::fmt;

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
        // (Part 2): return a useful, human-readable message for every
        // variant. Include the ids and numbers the variant carries.
        match self {
            LibraryError::EmptyTitle => write!(formatter, "Error: Empty Tittle not allowed"),

            LibraryError::DuplicateItemId { id } => {
                write!(formatter, "Error: An  Item with Id: {id} already exists")
            }

            LibraryError::DuplicateMemberId { id } => {
                write!(formatter, "Error: A  member  of id: {id} already exists")
            }

            LibraryError::ItemNotFound { id } => {
                write!(formatter, "Error: Item of id: {id}, was not found")
            }

            LibraryError::MemberNotFound { id } => {
                write!(formatter, "Error: Member of id: {id}, was not found")
            }

            LibraryError::ItemAlreadyOnLoan { id, member_id } => write!(
                formatter,
                "Error: Item of id: {id} already on loan to member of id: {member_id}"
            ),

            LibraryError::ItemNotOnLoan { id } => {
                write!(formatter, "Error: Item of Id {id} is not on loan")
            }

            LibraryError::ItemIsLost { id } => write!(formatter, "Error: Item of id: {id} is lost"),

            LibraryError::BorrowLimitReached { member_id, limit } => write!(
                formatter,
                "Error: Member with id: {member_id} has reached the limit of {limit} borrowed items"
            ),

            LibraryError::InvalidReturnDay {
                day_borrowed,
                day_returned,
            } => write!(
                formatter,
                "Error: Invalid return day, day borrowed: {day_borrowed} and day returned is: {day_returned}"
            ),
        }
    }
}

impl std::error::Error for LibraryError {}
