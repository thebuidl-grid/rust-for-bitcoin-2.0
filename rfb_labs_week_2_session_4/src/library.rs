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

    pub fn items_by_author<'a>(&'a self, author: &str) -> Vec<&'a Item> {
        self.items
            .iter()
            .filter(|item| item.author == author)
            .collect()
    }

    pub fn available_items(&self) -> Vec<&Item> {
        self.items
            .iter()
            .filter(|item| matches!(item.status, crate::catalogue::LoanStatus::Available))
            .collect()
    }

    pub fn longest_loan_item(&self) -> Option<&Item> {
        self.items.iter().max_by_key(|item| item.loan_days())
    }

   pub fn checkout(
    &mut self,
    item_id: u32,
    member_id: u32,
    day: u32,
) -> Result<(), LibraryError> {
    // 1. Check that the item exists.
    let item_index = self
        .items
        .iter()
        .position(|item| item.id == item_id)
        .ok_or(LibraryError::ItemNotFound { id: item_id })?;

    // 2. Check that the member exists.
    let member_index = self
        .members
        .iter()
        .position(|member| member.id == member_id)
        .ok_or(LibraryError::MemberNotFound { id: member_id })?;

    // 3. Check the item's status.
    match self.items[item_index].status {
        crate::catalogue::LoanStatus::Lost => {
            return Err(LibraryError::ItemIsLost { id: item_id });
        }

        crate::catalogue::LoanStatus::OnLoan {
            member_id: current_member,
            ..
        } => {
            return Err(LibraryError::ItemAlreadyOnLoan {
                id: item_id,
                member_id: current_member,
            });
        }

        crate::catalogue::LoanStatus::Available => {}
    }

    // 4. Check the member's borrowing limit.
    if self.members[member_index].borrowed_item_ids.len() >= MAX_ITEMS_PER_MEMBER {
    return Err(LibraryError::BorrowLimitReached {
        member_id,
        limit: MAX_ITEMS_PER_MEMBER,
    });
}

    // 5. All validation passed — now mutate both.
    self.items[item_index].status =
        crate::catalogue::LoanStatus::OnLoan {
            member_id,
            day_borrowed: day,
        };

    self.members[member_index]
    .borrowed_item_ids
    .push(item_id);

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
        LoanStatus::Lost => {
            return Err(LibraryError::ItemIsLost { id: item_id });
        }

        LoanStatus::Available => {
            return Err(LibraryError::ItemNotOnLoan { id: item_id });
        }

        LoanStatus::OnLoan {
            member_id,
            day_borrowed,
        } => (member_id, day_borrowed),
    };

    let days_held = day.checked_sub(day_borrowed).ok_or(
        LibraryError::InvalidReturnDay {
            day_borrowed,
            day_returned: day,
        },
    )?;

    let fee = self.items[item_index].late_fee_cents(days_held);

    self.items[item_index].status = LoanStatus::Available;

    if let Some(member) = self.members.iter_mut().find(|member| member.id == member_id) {
        member.borrowed_item_ids.retain(|id| *id != item_id);
    }

    Ok(fee)
}
}
