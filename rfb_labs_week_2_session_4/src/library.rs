use crate::{LoanStatus, LoanTerms};
use crate::catalogue::Item;
use crate::error::LibraryError;
use crate::member::{self, Member};

pub const MAX_ITEMS_PER_MEMBER: usize = 3;

/// Owns every item and every member.
///
/// The fields are private because the library is responsible for keeping an
/// item's `LoanStatus` and a member's borrowed-id list in agreement. Callers
/// reach the data through the borrowing lookups below.
// TODO(Part 3): delete this attribute once your lookups actually read the
// fields. It is here only so the untouched starter crate compiles clean.
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct Library {
    items: Vec<Item>,
    members: Vec<Member>,
}

impl Library {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_item(&mut self, item: Item) -> Result<(), LibraryError> {
        // TODO(Part 3): move `item` into the library. Reject an empty title

        if item.title.is_empty() {
            Err(LibraryError::EmptyTitle)
        }else if self.items.iter().any(|i| i.id == item.id ) {
            Err(LibraryError::DuplicateItemId { id: item.id })
        }else {
            self.items.push(item);
            Ok(())
        }
    }

    pub fn register_member(&mut self, member: Member) -> Result<(), LibraryError> {
        // TODO(Part 3): move `member` in. Reject an id already registered.
        if self.members.iter().any(|m| m.id == member.id ) {
            Err(LibraryError::DuplicateMemberId { id: member.id })
        }else {
            self.members.push(member);
            Ok(())
        }
    }

    pub fn find_item(&self, id: u32) -> Option<&Item> {
        // TODO(Part 3): borrow from `self`; do not clone.
        let item = self.items.iter().find(|item| item.id == id);
        item
    }

    pub fn find_member(&self, id: u32) -> Option<&Member> {
        // TODO(Part 3)
        let member = self.members.iter().find(|member| member.id == id);
        member
    }

    pub fn items_by_author<'a>(&'a self, author: &str) -> Vec<&'a Item> {
        // TODO(Part 3): return references to all matching items.
        self.items.iter().filter(|item| item.author.as_str() == author).collect()
    }

    pub fn available_items(&self) -> Vec<&Item> {
        // TODO(Part 3)
        self.items.iter().filter(|item| item.status == LoanStatus::Available).collect()
    }

    pub fn longest_loan_item(&self) -> Option<&Item> {
        // TODO(Part 4): the item that may be kept longest, via `LoanTerms`.
        self.items.iter().max_by_key(|item| item.loan_days())
    }

    pub fn checkout(&mut self, item_id: u32, member_id: u32, day: u32) -> Result<(), LibraryError> {
        // TODO(Part 5): validate in the order given in ASSIGNMENT.md, then
        // update the item's status and the member's list together.
        let item_index = self.items.iter().position(|i| i.id == item_id).ok_or(LibraryError::ItemNotFound { id: item_id })?;
        let member_index = self.members.iter().position(|m| m.id == member_id).ok_or(LibraryError::MemberNotFound { id: member_id })?;

        match self.items[item_index].status {
            LoanStatus::Available => {},
            LoanStatus::Lost => return Err(LibraryError::ItemIsLost { id: item_id }),
            LoanStatus::OnLoan { member_id, .. } => return Err(LibraryError::ItemAlreadyOnLoan { id: item_id, member_id })
        };

        if self.members[member_index].borrowed_item_ids.len() >= MAX_ITEMS_PER_MEMBER {
            return Err(LibraryError::BorrowLimitReached { member_id, limit: MAX_ITEMS_PER_MEMBER });
        }

        self.items[item_index].status = LoanStatus::OnLoan {
            member_id, day_borrowed: day 
        };

        self.members[member_index].borrowed_item_ids.push(item_id);

        Ok(())
    }

    /// Returns the late fee owed, in cents.
    pub fn return_item(&mut self, item_id: u32, day: u32) -> Result<u32, LibraryError> {
        // TODO(Part 6): checked subtraction must return InvalidReturnDay.
        
        let item_index = self.items.iter().position(|i| i.id == item_id)
                                        .ok_or(LibraryError::ItemNotFound { id: item_id })?;

        let (member_id, day_borrowed) = match self.items[item_index].status {
            LoanStatus::Lost => return Err(LibraryError::ItemIsLost { id: item_id }),
            LoanStatus::Available => return Err(LibraryError::ItemNotOnLoan { id: item_id }),
            LoanStatus::OnLoan { member_id, day_borrowed } => (member_id, day_borrowed)
        };

        if day < day_borrowed {
            return Err(LibraryError::InvalidReturnDay { day_borrowed, day_returned: day });
        }

        let days_held = day - day_borrowed;
        let fee = self.items[item_index].late_fee_cents(days_held);

        if let Some(member_index) = self.members.iter().position(|m| m.id == member_id) {
            self.members[member_index].borrowed_item_ids.retain(|&id| id != item_id);
        };

        self.items[item_index].status = LoanStatus::Available;

        Ok(fee)
    }
}   
