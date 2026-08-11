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

        if self
            .items
            .iter()
            .find(|item_| item_.id == item.id)
            .is_some()
        {
            Err(LibraryError::DuplicateItemId { id: item.id })
        } else if item.title.is_empty() {
            Err(LibraryError::EmptyTitle)
        } else {
            self.items.push(item);
            Ok(())
        }
    }

    pub fn register_member(&mut self, member: Member) -> Result<(), LibraryError> {
        // TODO(Part 3): move `member` in. Reject an id already registered.
        if self.members.iter().any(|member_| member_.id == member.id) {
            Err(LibraryError::DuplicateMemberId { id: member.id })
        } else if member.name.is_empty() {
            Err(LibraryError::EmptyTitle)
        } else {
            self.members.push(member);
            Ok(())
        }
    }

    pub fn find_item(&self, id: u32) -> Option<&Item> {
        // TODO(Part 3): borrow from `self`; do not clone.
        self.items.iter().find(|item_| item_.id == id)
    }

    pub fn find_member(&self, id: u32) -> Option<&Member> {
        // TODO(Part 3)
        self.members.iter().find(|member_| member_.id == id)
    }

    pub fn filter_items<F>(&self, predicate: F) -> Vec<&Item>
    where
        F: Fn(&Item) -> bool,
    {
        self.items.iter().filter(|item| predicate(item)).collect()
    }

    pub fn items_by_author<'a>(&'a self, author: &str) -> Vec<&'a Item> {
        // TODO(Part 3): return references to all matching items.
        self.filter_items(|item| item.author == author)
    }

    pub fn available_items(&self) -> Vec<&Item> {
        // TODO(Part 3)
        self.filter_items(|item| item.status == LoanStatus::Available)
    }

    pub fn longest_loan_item(&self) -> Option<&Item> {
        // TODO(Part 4): the item that may be kept longest, via `LoanTerms`.
        self.items.iter().find(|item_| item_.loan_days() == 21)
    }

    pub fn checkout(&mut self, item_id: u32, member_id: u32, day: u32) -> Result<(), LibraryError> {
        // TODO(Part 5): validate in the order given in ASSIGNMENT.md, then
        // update the item's status and the member's list together.

        let item = self.items.iter_mut().find(|item_| item_.id == item_id);
        let member = self
            .members
            .iter_mut()
            .find(|member_| member_.id == member_id);

        if item.is_none() {
            return Err(LibraryError::ItemNotFound { id: item_id });
        }

        if member.is_none() {
            return Err(LibraryError::MemberNotFound { id: member_id });
        }

        match item.as_ref().unwrap().status {
            LoanStatus::Lost => {
                Err(LibraryError::ItemIsLost { id: item_id })
            }
            LoanStatus::OnLoan { member_id, .. } => {
                Err(LibraryError::ItemAlreadyOnLoan {
                    id: item_id,
                    member_id,
                })
            }
            LoanStatus::Available => {
                if member.as_ref().unwrap().borrowed_item_ids.len() >= 3 {
                    Err(LibraryError::BorrowLimitReached {
                        member_id,
                        limit: MAX_ITEMS_PER_MEMBER,
                    })
                } else {
                    item.unwrap().status = LoanStatus::OnLoan {
                        member_id,
                        day_borrowed: day,
                    };
                    member.unwrap().borrowed_item_ids.push(item_id);

                    Ok(())
                }
            }
        }
    }

    /// Returns the late fee owed, in cents.
    pub fn return_item(&mut self, item_id: u32, day: u32) -> Result<u32, LibraryError> {
        // TODO(Part 6): checked subtraction must return InvalidReturnDay.
        let item = self.items.iter_mut().find(|item_| item_.id == item_id);

        let item_ref = &item.as_ref().unwrap();

        if item.is_none() {
            return Err(LibraryError::ItemNotFound { id: item_id });
        }

        match item_ref.status {
            LoanStatus::Lost => Err(LibraryError::ItemIsLost { id: item_id }),
            LoanStatus::Available => Err(LibraryError::ItemNotOnLoan { id: item_id }),
            LoanStatus::OnLoan {
                day_borrowed,
                member_id,
            } => {
                if day < day_borrowed {
                    return Err(LibraryError::InvalidReturnDay {
                        day_borrowed,
                        day_returned: day,
                    });
                }

                let late_fee = if item_ref.loan_days() < (day - day_borrowed) {
                    let extra_days = (day - day_borrowed) - item_ref.loan_days();
                    item_ref.late_fee_cents(extra_days)
                } else {
                    0
                };

                item.unwrap().status = LoanStatus::Available;

                let member = self
                    .members
                    .iter_mut()
                    .find(|member_| member_.id == member_id)
                    .unwrap();
                member.borrowed_item_ids.retain(|id| *id != item_id);
                member.borrowed_item_ids.retain(|ids| *ids != item_id);
                Ok(late_fee)
            }
        }
    }
}
