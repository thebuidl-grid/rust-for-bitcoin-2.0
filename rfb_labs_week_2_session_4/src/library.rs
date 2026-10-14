use crate::catalogue::{Item, LoanStatus, LoanTerms};
use crate::error::LibraryError;
use crate::member::Member;

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
        // and an id that is already stocked.
        // let _ = item;
        // todo!("add an item")

        if item.title.trim().is_empty() {
            return Err(LibraryError::EmptyTitle);
        }
        if self.items.iter().any(|i| i.id == item.id) {
            return Err(LibraryError::DuplicateItemId { id: item.id });
        }
        self.items.push(item);
        Ok(())
    }

    pub fn register_member(&mut self, member: Member) -> Result<(), LibraryError> {
        // TODO(Part 3): move `member` in. Reject an id already registered.
        // let _ = member;
        // todo!("register a member")

        if self.members.iter().any(|m| m.id == member.id) {
            return Err(LibraryError::DuplicateMemberId { id: member.id });
        }
        self.members.push(member);
        Ok(())
    }

    pub fn find_item(&self, id: u32) -> Option<&Item> {
        // TODO(Part 3): borrow from `self`; do not clone.
        // let _ = id;
        // todo!("find an item")

        self.items.iter().find(|i| i.id == id)
    }

    pub fn find_member(&self, id: u32) -> Option<&Member> {
        // TODO(Part 3)
        // let _ = id;
        // todo!("find a member")
        self.members.iter().find(|m| m.id == id)
    }

    pub fn items_by_author<'a>(&'a self, author: &str) -> Vec<&'a Item> {
        // TODO(Part 3): return references to all matching items.
        // let _ = author;
        // todo!("find items by author")

        self.filter_items(|item| item.author == author)
    }

    pub fn available_items(&self) -> Vec<&Item> {
        // TODO(Part 3)
        // let _ = ();
        // todo!("find the available items")
        self.filter_items(|item| matches!(item.status, LoanStatus::Available))
    }

    pub fn longest_loan_item(&self) -> Option<&Item> {
        // TODO(Part 4): the item that may be kept longest, via `LoanTerms`.
        // todo!("find the longest-loan item")
        self.items.iter().max_by_key(|i| i.loan_days())
    }

    pub fn checkout(&mut self, item_id: u32, member_id: u32, day: u32) -> Result<(), LibraryError> {
        // TODO(Part 5): validate in the order given in ASSIGNMENT.md, then
        // update the item's status and the member's list together.
        // let _ = (item_id, member_id, day);
        // todo!("check an item out")

        // 1. Verify item exists
        let item_idx = self
            .items
            .iter()
            .position(|i| i.id == item_id)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;

        // 2. Verify member exists
        let member_idx = self
            .members
            .iter()
            .position(|m| m.id == member_id)
            .ok_or(LibraryError::MemberNotFound { id: member_id })?;

        // 3. Verify item status availability
        match self.items[item_idx].status {
            LoanStatus::Available => {}
            LoanStatus::OnLoan {
                member_id: borrower_id,
                ..
            } => {
                return Err(LibraryError::ItemAlreadyOnLoan {
                    id: item_id,
                    member_id: borrower_id,
                });
            }
            LoanStatus::Lost => {
                return Err(LibraryError::ItemIsLost { id: item_id });
            }
        }

        // 4. Verify member's borrow limit
        if self.members[member_idx].borrowed_item_ids.len() >= MAX_ITEMS_PER_MEMBER {
            return Err(LibraryError::BorrowLimitReached {
                member_id,
                limit: MAX_ITEMS_PER_MEMBER,
            });
        }

        // Apply updates synchronously
        self.items[item_idx].status = LoanStatus::OnLoan {
            member_id,
            day_borrowed: day,
        };
        self.members[member_idx].borrowed_item_ids.push(item_id);

        Ok(())
    }

    /// Returns the late fee owed, in cents.
    pub fn return_item(&mut self, item_id: u32, day: u32) -> Result<u32, LibraryError> {
        // TODO(Part 6): checked subtraction must return InvalidReturnDay.
        // let _ = (item_id, day);
        // todo!("return an item")

        // 1. Find item
        let item = self
            .items
            .iter_mut()
            .find(|i| i.id == item_id)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;

        // 2. Extract borrow state or fail
        let (borrower_id, day_borrowed) = match item.status {
            LoanStatus::OnLoan {
                member_id,
                day_borrowed,
            } => (member_id, day_borrowed),
            LoanStatus::Available => return Err(LibraryError::ItemNotOnLoan { id: item_id }),
            LoanStatus::Lost => return Err(LibraryError::ItemIsLost { id: item_id }),
        };

        // 3. Validate return date relative to borrow date using checked subtraction
        if day < day_borrowed {
            return Err(LibraryError::InvalidReturnDay {
                day_borrowed,
                day_returned: day,
            });
        }

        // Calculate fees using LoanTerms trait logic
        let days_held = day - day_borrowed;
        let fee = item.late_fee_cents(days_held);

        // Update item status back to Available
        item.status = LoanStatus::Available;

        // Sync member's borrowed list
        if let Some(member) = self.members.iter_mut().find(|m| m.id == borrower_id) {
            member.borrowed_item_ids.retain(|&id| id != item_id);
        }

        Ok(fee)
    }

    /// Generic search helper that filters library items using a predicate closure.
    pub fn filter_items<P>(&self, predicate: P) -> Vec<&Item>
    where
        P: Fn(&Item) -> bool,
    {
        self.items.iter().filter(|item| predicate(item)).collect()
    }
}
