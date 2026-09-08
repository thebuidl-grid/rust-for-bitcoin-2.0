use crate::catalogue::Item;
use crate::error::LibraryError;
use crate::member::Member;
use crate::{LoanStatus, LoanTerms};

pub const MAX_ITEMS_PER_MEMBER: usize = 3;

/// Owns every item and every member.
///
/// The fields are private because the library is responsible for keeping an
/// item's `LoanStatus` and a member's borrowed-id list in agreement. Callers
/// reach the data through the borrowing lookups below.
// (Part 3): delete this attribute once your lookups actually read the
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
        // (Part 3): move `item` into the library. Reject an empty title
        // and an id that is already stocked.
        // if item.title.is_empty(){
        //     return Err(LibraryError::EmptyTitle);
        // }

        // if self.items.contains(&item){
        //     return  Err(LibraryError::DuplicateItemId { id: item.id });
        // }

        // Ok(())

        if item.title.is_empty() {
            return Err(LibraryError::EmptyTitle);
        }

        if self.items.iter().any(|items| items.id == item.id) {
            return Err(LibraryError::DuplicateItemId { id: item.id });
        }

        self.items.push(item);
        Ok(())
    }

    pub fn register_member(&mut self, member: Member) -> Result<(), LibraryError> {
        // (Part 3): move `member` in. Reject an id already registered.

        if self.members.iter().any(|members| members.id == member.id) {
            return Err(LibraryError::DuplicateMemberId { id: member.id });
        }

        self.members.push(member);
        Ok(())
    }

    pub fn find_item(&self, id: u32) -> Option<&Item> {
        // (Part 3): borrow from `self`; do not clone.

        self.items.iter().find(|item| item.id == id)
    }

    pub fn find_member(&self, id: u32) -> Option<&Member> {
        // (Part 3)

        self.members.iter().find(|member| member.id == id)
    }

    //part 9 added here for extra work
    pub fn filter_items<F>(&self, predicate: F) -> Vec<&Item>
    where
        F: Fn(&Item) -> bool,
    {
        self.items.iter().filter(|item| predicate(item)).collect()
    }

    pub fn items_by_author<'a>(&'a self, author: &str) -> Vec<&'a Item> {
        // (Part 3): return references to all matching items.
        //let mut matching_items = Vec::new();

        // self.items.iter().filter(|item| item.author == author).collect()

        self.filter_items(|item| item.author == author)
    }

    pub fn available_items(&self) -> Vec<&Item> {
        // (Part 3)

        // self.items.iter().filter(|item| item.status == LoanStatus::Available).collect()
        self.filter_items(|item| item.status == LoanStatus::Available)
    }

    pub fn longest_loan_item(&self) -> Option<&Item> {
        // (Part 4): the item that may be kept longest, via `LoanTerms`.
        self.items.iter().max_by_key(|item| item.loan_days())
    }

    pub fn checkout(&mut self, item_id: u32, member_id: u32, day: u32) -> Result<(), LibraryError> {
        // (Part 5): validate in the order given in ASSIGNMENT.md, then
        // update the item's status and the member's list together.
        let item_status = self
            .items
            .iter()
            .find(|i| i.id == item_id)
            .map(|i| i.status)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;

        if !self.members.iter().any(|member| member.id == member_id) {
            return Err(LibraryError::MemberNotFound { id: member_id });
        }
        match item_status {
            LoanStatus::Lost => return Err(LibraryError::ItemIsLost { id: item_id }),
            LoanStatus::OnLoan { .. } => {
                return Err(LibraryError::ItemAlreadyOnLoan {
                    id: item_id,
                    member_id,
                });
            }
            LoanStatus::Available => {}
        }

        if let Some(member) = self.members.iter().find(|member| member.id == member_id)
            && member.borrowed_item_ids.len() >= MAX_ITEMS_PER_MEMBER
        {
            return Err(LibraryError::BorrowLimitReached {
                member_id,
                limit: MAX_ITEMS_PER_MEMBER,
            });
        }

        //self.items.retain(|item|item != self.find_item(item_id).unwrap());

        if let Some(item) = self.items.iter_mut().find(|item| item.id == item_id) {
            item.status = LoanStatus::OnLoan {
                member_id,
                day_borrowed: day,
            };
        }
        self.members
            .iter_mut()
            .find(|member| member.id == member_id)
            .unwrap()
            .borrowed_item_ids
            .push(item_id);

        Ok(())
    }

    /// Returns the late fee owed, in cents.
    pub fn return_item(&mut self, item_id: u32, day: u32) -> Result<u32, LibraryError> {
        // (Part 6): checked subtraction must return InvalidReturnDay.
        let status = self
            .items
            .iter()
            .find(|item| item.id == item_id)
            .map(|item| item.status)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;

        let (member_id, day_borrowed) = match status {
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

        let fee = self
            .items
            .iter()
            .find(|item| item.id == item_id)
            .map(|item| item.late_fee_cents(days_held))
            .unwrap();

        if let Some(item) = self.items.iter_mut().find(|item| item.id == item_id) {
            item.status = LoanStatus::Available;
        }

        if let Some(member) = self
            .members
            .iter_mut()
            .find(|member| member.id == member_id)
        {
            member.borrowed_item_ids.retain(|&id| id != item_id);
        }

        Ok(fee)
    }
}
