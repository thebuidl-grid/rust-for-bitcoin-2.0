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
        //Reject duplicate id
        if self.items.iter().any(|i| i.id == item.id) {
            return Err(LibraryError::DuplicateItemId { id: item.id });
        }
        // Take ownership and store the item
        self.items.push(item);
        Ok(())
    }

    pub fn register_member(&mut self, member: Member) -> Result<(), LibraryError> {
        // TODO(Part 3): move `member` in. Reject an id already registered.
        let _ = member;
        if self.members.iter().any(|m| m.id == member.id) {
            return Err(LibraryError::DuplicateMemberId { id: member.id });
        }
        //take ownership and store the member
        self.members.push(member);
        Ok(())
    }

    pub fn find_item(&self, id: u32) -> Option<&Item> {
        // TODO(Part 3): borrow from `self`; do not clone.
        let _ = id;
        //borrow from self, do not clone
        self.items.iter().find(|item| item.id == id)
    }

    pub fn find_member(&self, id: u32) -> Option<&Member> {
        // TODO(Part 3)
        let _ = id;
        //borrow self do not clone
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
        // checking if item exists
        let item = self.items.iter().find(|i| i.id == item_id);
        if item.is_none() {
            return Err(LibraryError::ItemNotFound { id: item_id });
        }
        let item = item.unwrap();

        // check if member exists
        let member = self.members.iter().find(|m| m.id == member_id);
        if member.is_none() {
            return Err(LibraryError::MemberNotFound { id: member_id });
        }
        let member = member.unwrap();

        //check if item is lost
        if item.status == LoanStatus::Lost {
            return Err(LibraryError::ItemIsLost { id: item_id });
        }

        //check item is not already on loan
        if let LoanStatus::OnLoan {
            member_id: borrower,
            ..
        } = item.status
        {
            return Err(LibraryError::ItemAlreadyOnLoan {
                id: item_id,
                member_id: borrower,
            });
        }

        //check member hasn't reached borrow limit
        const MAX_ITEMS_PER_MEMBER: usize = 3;
        if member.borrowed_item_ids.len() >= MAX_ITEMS_PER_MEMBER {
            return Err(LibraryError::BorrowLimitReached {
                member_id,
                limit: MAX_ITEMS_PER_MEMBER,
            });
        }
        let item = self.items.iter_mut().find(|i| i.id == item_id).unwrap();
        item.status = LoanStatus::OnLoan {
            member_id,
            day_borrowed: day,
        };

        //find the member again mutably
        let member = self.members.iter_mut().find(|m| m.id == member_id).unwrap();
        member.borrowed_item_ids.push(item_id);

        Ok(())
    }

    /// Returns the late fee owed, in cents.
    pub fn return_item(&mut self, item_id: u32, day: u32) -> Result<u32, LibraryError> {
        // TODO(Part 6): checked subtraction must return InvalidReturnDay.
        let _ = (item_id, day);
        let item = self.items.iter().find(|i| i.id == item_id);
        if item.is_none() {
            return Err(LibraryError::ItemNotFound { id: item_id });
        }

        let item = item.unwrap();

        //check item is not lost
        if item.status == LoanStatus::Lost {
            return Err(LibraryError::ItemIsLost { id: item_id });
        }

        //check item is actually on loan and extract borrow details
        let (member_id, day_borrowed) = match item.status {
            LoanStatus::OnLoan {
                member_id,
                day_borrowed,
            } => (member_id, day_borrowed),
            _ => return Err(LibraryError::ItemNotOnLoan { id: item_id }),
        };
        //validate return day is not before borrow day using checked subtraction
        let days_held = day
            .checked_sub(day_borrowed)
            .ok_or(LibraryError::InvalidReturnDay {
                day_borrowed,
                day_returned: day,
            })?;

        //calculate late fee usiing loanterms
        let fee = item.late_fee_cents(days_held);
        let item = self.items.iter_mut().find(|i| i.id == item_id).unwrap();
        item.status = LoanStatus::Available;

        //Remove item_id from member's borrowed list
        let member = self.members.iter_mut().find(|m| m.id == member_id).unwrap();
        member.borrowed_item_ids.retain(|&id| id != item_id);
        Ok(fee)
    }
}
