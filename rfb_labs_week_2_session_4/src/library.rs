use crate::error::LibraryError;
use crate::member::Member;
use crate::catalogue::{Item, LoanStatus, LoanTerms};


pub const MAX_ITEMS_PER_MEMBER: usize = 3;

/// Owns every item and every member.
///
/// The fields are private because the library is responsible for keeping an
/// item's `LoanStatus` and a member's borrowed-id list in agreement. Callers
/// reach the data through the borrowing lookups below.
// TODO(Part 3): delete this attribute once your lookups actually read the
// fields. It is here only so the untouched starter crate compiles clean.
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
        // and an id that is already stocked.
        let _ = item;
        if item.title.is_empty() {
            return Err(LibraryError::EmptyTitle);
        }
        if self.items.iter().any(|existing| existing.id == item.id) {
            return Err(LibraryError::DuplicateItemId { id: item.id });
        }
        self.items.push(item);
        Ok(())
    }

    pub fn register_member(&mut self, member: Member) -> Result<(), LibraryError> {
        // TODO(Part 3): move `member` in. Reject an id already registered.
        let _ = member;
        if self.members.iter().any(|existing| existing.id == member.id) {
            return Err(LibraryError::DuplicateMemberId { id: member.id });
        }
        self.members.push(member);
        Ok(())
    }

    pub fn find_item(&self, id: u32) -> Option<&Item> {
        // TODO(Part 3): borrow from `self`; do not clone.
        let _ = id;
        self.items.iter().find(|item| item.id == id)
    }

    pub fn find_member(&self, id: u32) -> Option<&Member> {
        // TODO(Part 3)
        let _ = id;
        self.members.iter().find(|member| member.id == id)
    }

    pub fn items_by_author<'a>(&'a self, author: &str) -> Vec<&'a Item> {
        // TODO(Part 3): return references to all matching items.
        let _ = author;
        self.items
            .iter()
            .filter(|item| item.author == author)
            .collect()
    }

    pub fn available_items(&self) -> Vec<&Item> {
        // TODO(Part 3)
        self.items
            .iter()
            .filter(|item| item.status == LoanStatus::Available)
            .collect()
    }

    pub fn longest_loan_item(&self) -> Option<&Item> {
        // TODO(Part 4): the item that may be kept longest, via `LoanTerms`.
       self.items.iter().max_by_key(|item| item.loan_days())
    }

    pub fn checkout(&mut self, item_id: u32, member_id: u32, day: u32) -> Result<(), LibraryError> {
        // TODO(Part 5): validate in the order given in ASSIGNMENT.md, then
        // update the item's status and the member's list together.
        let _ = (item_id, member_id, day);
        let item = self
            .find_item(item_id)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;
        let member = self
            .find_member(member_id)
            .ok_or(LibraryError::MemberNotFound { id: member_id })?;

        match item.status {
            LoanStatus::Lost => return Err(LibraryError::ItemIsLost { id: item_id }),
            LoanStatus::OnLoan { member_id: borrower, .. } => {
                return Err(LibraryError::ItemAlreadyOnLoan {
                    id: item_id,
                    member_id: borrower,
                })
            }
            LoanStatus::Available => {}
        }

        if member.borrowed_item_ids.len() >= MAX_ITEMS_PER_MEMBER {
            return Err(LibraryError::BorrowLimitReached {
                member_id,
                limit: MAX_ITEMS_PER_MEMBER,
            });
        }

        // Validation done, and `item`/`member` (both immutable borrows) are no
        // longer used past this point — so these two mutable borrows are each
        // free to happen, one at a time, never overlapping.
        let item = self.items.iter_mut().find(|item| item.id == item_id).unwrap();
        item.status = LoanStatus::OnLoan {
            member_id,
            day_borrowed: day,
        };

        let member = self.members.iter_mut().find(|member| member.id == member_id).unwrap();
        member.borrowed_item_ids.push(item_id);

        Ok(())
    }

    /// Returns the late fee owed, in cents.
    pub fn return_item(&mut self, item_id: u32, day: u32) -> Result<u32, LibraryError> {
        // TODO(Part 6): checked subtraction must return InvalidReturnDay.
        let _ = (item_id, day);
         let item = self
            .find_item(item_id)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;

        let (member_id, day_borrowed) = match item.status {
            LoanStatus::Lost => return Err(LibraryError::ItemIsLost { id: item_id }),
            LoanStatus::Available => return Err(LibraryError::ItemNotOnLoan { id: item_id }),
            LoanStatus::OnLoan {
                member_id,
                day_borrowed,
            } => (member_id, day_borrowed),
        };

        let days_held = day
            .checked_sub(day_borrowed)
            .ok_or(LibraryError::InvalidReturnDay {
                day_borrowed,
                day_returned: day,
            })?;

        let fee = item.late_fee_cents(days_held);

        let item = self.items.iter_mut().find(|item| item.id == item_id).unwrap();
        item.status = LoanStatus::Available;

        let member = self.members.iter_mut().find(|member| member.id == member_id).unwrap();
        member.borrowed_item_ids.retain(|&id| id != item_id);

        Ok(fee)
    }
}
