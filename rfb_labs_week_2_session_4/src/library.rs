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

    pub fn add_item(&mut self, item: Item) -> Result<(), LibraryError> {
        if item.title.is_empty() {
            return Err(LibraryError::EmptyTitle);
        }
        if self.find_item(item.id).is_some() {
            return Err(LibraryError::DuplicateItemId { id: item.id });
        }
        self.items.push(item);
        Ok(())
    }

    pub fn register_member(&mut self, member: Member) -> Result<(), LibraryError> {
        if self.find_member(member.id).is_some() {
            return Err(LibraryError::DuplicateMemberId { id: member.id });
        }
        self.members.push(member);
        Ok(())
    }

    pub fn find_item(&self, id: u32) -> Option<&Item> {
        self.items.iter().find(|item| item.id == id)
    }

    pub fn find_member(&self, id: u32) -> Option<&Member> {
        self.members.iter().find(|member| member.id == id)
    }

    pub fn items_by_author<'a>(&'a self, author: &str) -> Vec<&'a Item> {
        self.filter_items(|item| item.author == author)
    }

    pub fn available_items(&self) -> Vec<&Item> {
        self.filter_items(|item| item.status == LoanStatus::Available)
    }

    /// Part 9 (optional): a general search, expressed once so the two
    /// lookups above can be written in terms of it instead of repeating the
    /// same `iter().filter().collect()` shape.
    pub fn filter_items<F>(&self, predicate: F) -> Vec<&Item>
    where
        F: Fn(&Item) -> bool,
    {
        self.items.iter().filter(|item| predicate(item)).collect()
    }

    pub fn longest_loan_item(&self) -> Option<&Item> {
        self.items.iter().max_by_key(|item| item.loan_days())
    }

    pub fn checkout(&mut self, item_id: u32, member_id: u32, day: u32) -> Result<(), LibraryError> {
        // Validate first, in the order the assignment requires.
        let item = self
            .find_item(item_id)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;

        if self.find_member(member_id).is_none() {
            return Err(LibraryError::MemberNotFound { id: member_id });
        }

        match item.status {
            LoanStatus::Lost => return Err(LibraryError::ItemIsLost { id: item_id }),
            LoanStatus::OnLoan {
                member_id: borrower,
                ..
            } => {
                return Err(LibraryError::ItemAlreadyOnLoan {
                    id: item_id,
                    member_id: borrower,
                });
            }
            LoanStatus::Available => {}
        }

        let member = self.find_member(member_id).expect("checked above");
        if member.borrowed_item_ids.len() >= MAX_ITEMS_PER_MEMBER {
            return Err(LibraryError::BorrowLimitReached {
                member_id,
                limit: MAX_ITEMS_PER_MEMBER,
            });
        }

        // Mutate second: item and member borrows never overlap.
        let item = self
            .items
            .iter_mut()
            .find(|item| item.id == item_id)
            .expect("checked above");
        item.status = LoanStatus::OnLoan {
            member_id,
            day_borrowed: day,
        };

        let member = self
            .members
            .iter_mut()
            .find(|member| member.id == member_id)
            .expect("checked above");
        member.borrowed_item_ids.push(item_id);

        Ok(())
    }

    /// Returns the late fee owed, in cents.
    pub fn return_item(&mut self, item_id: u32, day: u32) -> Result<u32, LibraryError> {
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

        let item = self
            .items
            .iter_mut()
            .find(|item| item.id == item_id)
            .expect("checked above");
        item.status = LoanStatus::Available;

        let member = self
            .members
            .iter_mut()
            .find(|member| member.id == member_id)
            .expect("member existed at checkout time");
        member.borrowed_item_ids.retain(|&id| id != item_id);

        Ok(fee)
    }
}
