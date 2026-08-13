use crate::catalogue::{Item, LoanStatus, LoanTerms};
use crate::error::LibraryError;
use crate::member::Member;

pub const MAX_ITEMS_PER_MEMBER: usize = 3;

/// Owns every item and every member.
///
/// The fields are private because the library is responsible for keeping an
/// item's `LoanStatus` and a member's borrowed-id list in agreement. Callers
/// reach the data through the borrowing lookups below.
#[derive(Debug, Default)]
pub struct Library {
    items: Vec<Item>,
    members: Vec<Member>,
}

impl Library {
    pub fn new() -> Self {
        Self::default()
    }

    // === Ownership methods

    pub fn add_item(&mut self, item: Item) -> Result<(), LibraryError> {
        if item.title.is_empty() {
            return Err(LibraryError::EmptyTitle);
        }
        if self.items.iter().any(|i| i.id == item.id) {
            return Err(LibraryError::DuplicateItemId { id: item.id });
        }
        self.items.push(item);
        Ok(())
    }

    pub fn register_member(&mut self, member: Member) -> Result<(), LibraryError> {
        if self.members.iter().any(|m| m.id == member.id) {
            return Err(LibraryError::DuplicateMemberId { id: member.id });
        }
        self.members.push(member);
        Ok(())
    }

    // === Borrowing lookups

    pub fn find_item(&self, id: u32) -> Option<&Item> {
        self.items.iter().find(|i| i.id == id)
    }

    pub fn find_member(&self, id: u32) -> Option<&Member> {
        self.members.iter().find(|m| m.id == id)
    }

    pub fn filter_items<F>(&self, predicate: F) -> Vec<&Item>
    where
        F: Fn(&Item) -> bool,
    {
        self.items.iter().filter(|i| predicate(i)).collect()
    }

    pub fn items_by_author<'a>(&'a self, author: &str) -> Vec<&'a Item> {
        self.filter_items(|i| i.author == author)
    }

    pub fn available_items(&self) -> Vec<&Item> {
        self.filter_items(|i| i.status == LoanStatus::Available)
    }

    // === Trait-based query

    pub fn longest_loan_item(&self) -> Option<&Item> {
        self.items.iter().max_by_key(|i| i.loan_days())
    }

    // === Checkout

    pub fn checkout(&mut self, item_id: u32, member_id: u32, day: u32) -> Result<(), LibraryError> {
        // Validate using shared refs before any mutation.
        let item = self
            .items
            .iter()
            .find(|i| i.id == item_id)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;

        self.members
            .iter()
            .find(|m| m.id == member_id)
            .ok_or(LibraryError::MemberNotFound { id: member_id })?;

        match item.status {
            LoanStatus::Lost => return Err(LibraryError::ItemIsLost { id: item_id }),
            LoanStatus::OnLoan {
                member_id: borrower,
                ..
            } => {
                return Err(LibraryError::ItemAlreadyOnLoan {
                    id: item_id,
                    member_id: borrower,
                })
            }
            LoanStatus::Available => {}
        }

        let member = self.members.iter().find(|m| m.id == member_id).unwrap();
        if member.borrowed_item_ids.len() >= MAX_ITEMS_PER_MEMBER {
            return Err(LibraryError::BorrowLimitReached {
                member_id,
                limit: MAX_ITEMS_PER_MEMBER,
            });
        }

        // Validation passed — now mutate using index-based access so we hold
        // no borrows across the two mutable updates.
        let item_idx = self.items.iter().position(|i| i.id == item_id).unwrap();
        self.items[item_idx].status = LoanStatus::OnLoan {
            member_id,
            day_borrowed: day,
        };

        let member_idx = self
            .members
            .iter()
            .position(|m| m.id == member_id)
            .unwrap();
        self.members[member_idx].borrowed_item_ids.push(item_id);

        Ok(())
    }

    // === Return

    /// Returns the late fee owed, in cents.
    pub fn return_item(&mut self, item_id: u32, day: u32) -> Result<u32, LibraryError> {
        let item = self
            .items
            .iter()
            .find(|i| i.id == item_id)
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

        // Validation done — mutate with index access.
        let item_idx = self.items.iter().position(|i| i.id == item_id).unwrap();
        self.items[item_idx].status = LoanStatus::Available;

        if let Some(member_idx) = self.members.iter().position(|m| m.id == member_id) {
            self.members[member_idx]
                .borrowed_item_ids
                .retain(|&id| id != item_id);
        }

        Ok(fee)
    }
}

