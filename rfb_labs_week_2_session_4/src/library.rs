use crate::catalogue::{Item, LoanTerms};
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
              // TODO(Part 3): move `item` into the library. Reject an empty title
        // and an id that is already stocked.
        let _ = item;
        // todo!("add an item")
        // let _ = item;
        // todo!("add an item")
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
         // TODO(Part 3): move `member` in. Reject an id already registered.
        let _ = member;
        // todo!("register a member")
        
        if self.members.iter().any(|m| m.id == member.id) {
            return Err(LibraryError::DuplicateMemberId { id: member.id });
        }
        self.members.push(member);
        Ok(())
    }

    pub fn find_item(&self, id: u32) -> Option<&Item> {
        // TODO(Part 3): borrow from `self`; do not clone.
        let _ = id;
        // todo!("find an item")
        self.items.iter().find(|item| item.id == id)
    }

    pub fn find_member(&self, id: u32) -> Option<&Member> {
        // TODO(Part 3)
        let _ = id;
        // todo!("find a member")
        self.members.iter().find(|member| member.id == id)
    }

    pub fn items_by_author<'a>(&'a self, author: &str) -> Vec<&'a Item> {
        self.items
            .iter()
            .filter(|item| item.author == author)
            .collect()
    }

    pub fn available_items(&self) -> Vec<&Item> {
        use crate::catalogue::LoanStatus;
        self.items
            .iter()
            .filter(|item| item.status == LoanStatus::Available)
            .collect()
    }

    pub fn longest_loan_item(&self) -> Option<&Item> {
         // TODO(Part 4): the item that may be kept longest, via `LoanTerms`.
        // todo!("find the longest-loan item")
        self.items.iter().max_by_key(|item| item.loan_days())
    }

    pub fn checkout(&mut self, item_id: u32, member_id: u32, day: u32) -> Result<(), LibraryError> {
         // TODO(Part 5): validate in the order given in ASSIGNMENT.md, then
        // update the item's status and the member's list together.
        let _ = (item_id, member_id, day);
        // todo!("check an item out")
        use crate::catalogue::LoanStatus;

        // Validate in order
        let item = self
            .find_item(item_id)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;

        let _member = self
            .find_member(member_id)
            .ok_or(LibraryError::MemberNotFound { id: member_id })?;

        match item.status {
            LoanStatus::Lost => return Err(LibraryError::ItemIsLost { id: item_id }),
            LoanStatus::OnLoan {
                member_id: existing_member,
                ..
            } => {
                return Err(LibraryError::ItemAlreadyOnLoan {
                    id: item_id,
                    member_id: existing_member,
                });
            }
            LoanStatus::Available => {}
        }

        let member = self.find_member(member_id).unwrap();
        if member.borrowed_item_ids.len() >= MAX_ITEMS_PER_MEMBER {
            return Err(LibraryError::BorrowLimitReached {
                member_id,
                limit: MAX_ITEMS_PER_MEMBER,
            });
        }

        // Mutate: find and update the item
        let item = self.items.iter_mut().find(|i| i.id == item_id).unwrap();
        item.status = LoanStatus::OnLoan {
            member_id,
            day_borrowed: day,
        };

        // Mutate: find and update the member
        let member = self.members.iter_mut().find(|m| m.id == member_id).unwrap();
        member.borrowed_item_ids.push(item_id);

        Ok(())
    }

    /// Returns the late fee owed, in cents.
    pub fn return_item(&mut self, item_id: u32, day: u32) -> Result<u32, LibraryError> {
         // TODO(Part 6): checked subtraction must return InvalidReturnDay.
        let _ = (item_id, day);
        // todo!("return an item")
        use crate::catalogue::LoanStatus;

        let item = self
            .find_item(item_id)
            .ok_or(LibraryError::ItemNotFound { id: item_id })?;

        match item.status {
            LoanStatus::Lost => Err(LibraryError::ItemIsLost { id: item_id }),
            LoanStatus::Available => Err(LibraryError::ItemNotOnLoan { id: item_id }),
            LoanStatus::OnLoan {
                member_id,
                day_borrowed,
            } => {
                let days_held =
                    day.checked_sub(day_borrowed)
                        .ok_or(LibraryError::InvalidReturnDay {
                            day_borrowed,
                            day_returned: day,
                        })?;

                let fee = self.find_item(item_id).unwrap().late_fee_cents(days_held);

                // Update the item status
                let item = self.items.iter_mut().find(|i| i.id == item_id).unwrap();
                item.status = LoanStatus::Available;

                // Remove from member's borrowed list
                let member = self.members.iter_mut().find(|m| m.id == member_id).unwrap();
                member.borrowed_item_ids.retain(|&id| id != item_id);

                Ok(fee)
            }
        }
    }
}
