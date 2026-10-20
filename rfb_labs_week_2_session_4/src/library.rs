use crate::catalogue::Item;
use crate::catalogue::LoanStatus;
use crate::catalogue::LoanTerms;
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
        for i in &self.items {
            if i.id == item.id {
                return Err(LibraryError::DuplicateItemId { id: item.id });
            }
        }
        self.items.push(item);

        Ok(())
    }

    pub fn register_member(&mut self, member: Member) -> Result<(), LibraryError> {
        for m in &self.members {
            if m.id == member.id {
                return Err(LibraryError::DuplicateMemberId { id: member.id });
            }
        }
        self.members.push(member);

        Ok(())
    }

    pub fn find_item(&self, id: u32) -> Option<&Item> {
        for item in &self.items {
            if item.id == id {
                return Some(item);
            }
        }
        None
    }

    pub fn find_member(&self, id: u32) -> Option<&Member> {
        // TODO(Part 3)
        for member in &self.members {
            if member.id == id {
                return Some(member);
            }
        }
        None
    }

    pub fn items_by_author<'a>(&'a self, author: &str) -> Vec<&'a Item> {
        self.filter_items(|item| item.author == author)
    }

    pub fn available_items(&self) -> Vec<&Item> {
        self.filter_items(|item| item.status == LoanStatus::Available)
    }

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
        let item = self
            .items
            .iter()
            .find(|item| item.id == item_id)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;

        self.members
            .iter()
            .find(|member| member.id == member_id)
            .ok_or(LibraryError::MemberNotFound { id: member_id })?;

        if item.status == LoanStatus::Lost {
            return Err(LibraryError::ItemIsLost { id: item_id });
        }

        if let LoanStatus::OnLoan { .. } = item.status {
            return Err(LibraryError::ItemAlreadyOnLoan {
                id: item_id,
                member_id,
            });
        }

        let current_loans = self
            .items
            .iter()
            .filter(
                |i| matches!(i.status, LoanStatus::OnLoan { member_id: m, .. } if m == member_id),
            )
            .count();

        if current_loans >= MAX_ITEMS_PER_MEMBER {
            return Err(LibraryError::BorrowLimitReached {
                member_id,
                limit: current_loans,
            });
        }

        let item = self.items.iter_mut().find(|i| i.id == item_id).unwrap();
        item.status = LoanStatus::OnLoan {
            member_id,
            day_borrowed: day,
        };

        let member = self.members.iter_mut().find(|m| m.id == member_id).unwrap();
        member.borrowed_item_ids.push(item_id);

        Ok(())
    }

    /// Returns the late fee owed, in cents.
    pub fn return_item(&mut self, item_id: u32, day: u32) -> Result<u32, LibraryError> {
        let item = self
            .items
            .iter_mut()
            .find(|item| item.id == item_id)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;

        let (member_id, day_borrowed) = match item.status {
            LoanStatus::OnLoan {
                member_id,
                day_borrowed,
            } => (member_id, day_borrowed),
            _ => return Err(LibraryError::ItemNotOnLoan { id: item_id }),
        };

        let days_held = day
            .checked_sub(day_borrowed)
            .ok_or(LibraryError::InvalidReturnDay {
                day_borrowed,
                day_returned: day,
            })?;

        let fee = item.late_fee_cents(days_held);

        item.status = LoanStatus::Available;

        let member = self
            .members
            .iter_mut()
            .find(|m| m.id == member_id)
            .ok_or(LibraryError::MemberNotFound { id: member_id })?;

        member.borrowed_item_ids.retain(|&id| id != item_id);

        Ok(fee)
    }
}
