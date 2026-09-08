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
        if item.title.trim().is_empty() {
            return Err(LibraryError::EmptyTitle);
        }
        if self.items.iter().any(|existing| existing.id == item.id) {
            return Err(LibraryError::DuplicateItemId { id: item.id });
        }

        self.items.push(item);
        Ok(())
    }

    pub fn register_member(&mut self, member: Member) -> Result<(), LibraryError> {
        if self.members.iter().any(|existing| existing.id == member.id) {
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

    /// Returns references to every item for which `predicate` returns `true`.
    pub fn filter_items<F>(&self, predicate: F) -> Vec<&Item>
    where
        F: Fn(&Item) -> bool,
    {
        self.items.iter().filter(|item| predicate(item)).collect()
    }

    pub fn items_by_author<'a>(&'a self, author: &str) -> Vec<&'a Item> {
        self.filter_items(|item| item.author == author)
    }

    pub fn available_items(&self) -> Vec<&Item> {
        self.filter_items(|item| item.status == LoanStatus::Available)
    }

    pub fn longest_loan_item(&self) -> Option<&Item> {
        self.items.iter().max_by_key(|item| item.loan_days())
    }

    pub fn checkout(&mut self, item_id: u32, member_id: u32, day: u32) -> Result<(), LibraryError> {
        let item_index = self
            .items
            .iter()
            .position(|item| item.id == item_id)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;

        let member_index = self
            .members
            .iter()
            .position(|member| member.id == member_id)
            .ok_or(LibraryError::MemberNotFound { id: member_id })?;

        match self.items[item_index].status {
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

        if self.members[member_index].borrowed_item_ids.len() >= MAX_ITEMS_PER_MEMBER {
            return Err(LibraryError::BorrowLimitReached {
                member_id,
                limit: MAX_ITEMS_PER_MEMBER,
            });
        }

        self.items[item_index].status = LoanStatus::OnLoan {
            member_id,
            day_borrowed: day,
        };
        self.members[member_index].borrowed_item_ids.push(item_id);

        Ok(())
    }

    /// Returns the late fee owed, in cents.
    pub fn return_item(&mut self, item_id: u32, day: u32) -> Result<u32, LibraryError> {
        let item_index = self
            .items
            .iter()
            .position(|item| item.id == item_id)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;

        let (member_id, day_borrowed) = match self.items[item_index].status {
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

        let fee = self.items[item_index].late_fee_cents(days_held);
        self.items[item_index].status = LoanStatus::Available;

        // A loan's `member_id` is only ever set by `checkout`, which requires
        // the member to already be registered, so this lookup cannot fail
        // without a bug elsewhere in `Library` — worth a panic, not a `Result`.
        let member_index = self
            .members
            .iter()
            .position(|member| member.id == member_id)
            .expect("a member on an active loan must be registered");
        self.members[member_index]
            .borrowed_item_ids
            .retain(|&id| id != item_id);

        Ok(fee)
    }
}
