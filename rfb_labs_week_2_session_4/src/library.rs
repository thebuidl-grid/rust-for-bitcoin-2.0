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
        if self.item_index(item.id).is_some() {
            return Err(LibraryError::DuplicateItemId { id: item.id });
        }

        self.items.push(item);
        Ok(())
    }

    pub fn register_member(&mut self, member: Member) -> Result<(), LibraryError> {
        if self.member_index(member.id).is_some() {
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

    /// Part 9: one borrowing search the specific lookups above are written in
    /// terms of. The closure only ever sees a `&Item`, so nothing is cloned
    /// and nothing can be mutated behind the library's back.
    pub fn filter_items<F>(&self, predicate: F) -> Vec<&Item>
    where
        F: Fn(&Item) -> bool,
    {
        self.items.iter().filter(|item| predicate(item)).collect()
    }

    pub fn longest_loan_item(&self) -> Option<&Item> {
        // `reduce` with a strict `>` keeps the first of several equal-length
        // loans, which makes the answer stable as the catalogue grows.
        self.items.iter().reduce(|longest, item| {
            if item.loan_days() > longest.loan_days() {
                item
            } else {
                longest
            }
        })
    }

    pub fn checkout(&mut self, item_id: u32, member_id: u32, day: u32) -> Result<(), LibraryError> {
        // Validate everything first, holding only indices rather than
        // references, so the two mutations below cannot half-apply.
        let item_index = self
            .item_index(item_id)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;
        let member_index = self
            .member_index(member_id)
            .ok_or(LibraryError::MemberNotFound { id: member_id })?;

        match self.items[item_index].status {
            LoanStatus::Lost => return Err(LibraryError::ItemIsLost { id: item_id }),
            LoanStatus::OnLoan {
                member_id: holder, ..
            } => {
                return Err(LibraryError::ItemAlreadyOnLoan {
                    id: item_id,
                    member_id: holder,
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

        // Past this point nothing can fail, so the item and the member are
        // guaranteed to move together.
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
            .item_index(item_id)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;

        // `LoanStatus` is `Copy`, so this ends the borrow of `self.items`
        // immediately and leaves the borrow the loan actually carries.
        let (member_id, day_borrowed) = match self.items[item_index].status {
            LoanStatus::Lost => return Err(LibraryError::ItemIsLost { id: item_id }),
            LoanStatus::Available => return Err(LibraryError::ItemNotOnLoan { id: item_id }),
            LoanStatus::OnLoan {
                member_id,
                day_borrowed,
            } => (member_id, day_borrowed),
        };

        // A clock that ran backwards is a caller error, not an underflow panic.
        let days_held = day
            .checked_sub(day_borrowed)
            .ok_or(LibraryError::InvalidReturnDay {
                day_borrowed,
                day_returned: day,
            })?;

        let fee = self.items[item_index].late_fee_cents(days_held);

        self.items[item_index].status = LoanStatus::Available;
        if let Some(member_index) = self.member_index(member_id) {
            self.members[member_index]
                .borrowed_item_ids
                .retain(|held| *held != item_id);
        }

        Ok(fee)
    }

    /// Positions, not references: a `usize` borrows nothing, so the caller
    /// stays free to take `&mut self` afterwards.
    fn item_index(&self, id: u32) -> Option<usize> {
        self.items.iter().position(|item| item.id == id)
    }

    fn member_index(&self, id: u32) -> Option<usize> {
        self.members.iter().position(|member| member.id == id)
    }
}
