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
        // todo!("add an item")
        if item.title.is_empty() {
            return Err(LibraryError::EmptyTitle);
        } else if self.items.contains(&item) {
            return Err(LibraryError::DuplicateItemId { id: item.id });
        };

        self.items.push(item);

        Ok(())
    }

    pub fn register_member(&mut self, member: Member) -> Result<(), LibraryError> {
        // TODO(Part 3): move `member` in. Reject an id already registered.
        // todo!("register a member")
        if self.members.contains(&member) {
            return Err(LibraryError::DuplicateMemberId { id: member.id });
        };
        self.members.push(member);
        Ok(())
    }

    pub fn find_item(&self, id: u32) -> Option<&Item> {
        // TODO(Part 3): borrow from `self`; do not clone.
        // todo!("find an item")
        self.items.iter().find(|item| item.id == id)
    }

    pub fn find_member(&self, id: u32) -> Option<&Member> {
        // TODO(Part 3)
        // todo!("find a member")
        self.members.iter().find(|member| member.id == id)
    }

    pub fn filter_items(&self, predicate: impl Fn(&Item) -> bool) -> Vec<&Item> {
        self.items.iter().filter(|item| predicate(item)).collect()
    }

    pub fn items_by_author(&self, author: &str) -> Vec<&Item> {
        self.filter_items(|item| item.author == author)
    }

    pub fn available_items(&self) -> Vec<&Item> {
        self.filter_items(|item| matches!(item.status, LoanStatus::Available))
    }

    pub fn longest_loan_item(&self) -> Option<&Item> {
        // TODO(Part 4): the item that may be kept longest, via `LoanTerms`.
        // todo!("find the longest-loan item")
        self.items
            .iter()
            .filter(|item| matches!(item.status, LoanStatus::OnLoan { .. }))
            .fold(None, |acc: Option<(u32, &Item)>, item| {
                let loan_days = item.loan_days();
                match acc {
                    Some((best, _)) if best >= loan_days => acc,
                    _ => Some((loan_days, item)),
                }
            })
            .map(|(_, item)| item)
    }

    pub fn checkout(&mut self, item_id: u32, member_id: u32, day: u32) -> Result<(), LibraryError> {
        // TODO(Part 5): validate in the order given in ASSIGNMENT.md, then
        // update the item's status and the member's list together.
        // let _ = (item_id, member_id, day);
        // todo!("check an item out")

        // unknown item
        if self.find_item(item_id).is_none() {
            return Err(LibraryError::ItemNotFound { id: item_id });
        };

        // unknown, fold it and try finding members ,
        if self.find_member(member_id).is_none() {
            return Err(LibraryError::MemberNotFound { id: member_id });
        };

        // item is lost/on loan
        if let Some(item) = self.find_item(item_id) {
            match item.status {
                LoanStatus::Lost => return Err(LibraryError::ItemIsLost { id: item_id }),
                LoanStatus::OnLoan {
                    member_id: holder_id,
                    ..
                } => {
                    return Err(LibraryError::ItemAlreadyOnLoan {
                        id: item_id,
                        member_id: holder_id,
                    });
                }
                _ => {}
            }
        }

        // borrow limit reached
        if self
            .find_member(member_id)
            .is_some_and(|member| member.borrowed_item_ids.len() >= MAX_ITEMS_PER_MEMBER)
        {
            return Err(LibraryError::BorrowLimitReached {
                member_id,
                limit: MAX_ITEMS_PER_MEMBER,
            });
        }

        let item = self
            .items
            .iter_mut()
            .find(|item| item.id == item_id)
            .unwrap();
        item.status = LoanStatus::OnLoan {
            member_id,
            day_borrowed: day,
        };
        let member = self
            .members
            .iter_mut()
            .find(|member| member.id == member_id)
            .unwrap();
        member.borrowed_item_ids.push(item_id);
        Ok(())
    }

    /// Returns the late fee owed, in cents.
    pub fn return_item(&mut self, item_id: u32, day: u32) -> Result<u32, LibraryError> {
        // TODO(Part 6): checked subtraction must return InvalidReturnDay.
        // let _ = (item_id, day); day is return_day2
        // todo!("return an item")
        let item = match self.find_item(item_id) {
            Some(item) => item,
            None => return Err(LibraryError::ItemNotFound { id: item_id }),
        };

        let (fee, member_id) = match item.status {
            LoanStatus::OnLoan {
                member_id,
                day_borrowed,
            } => {
                let days_held = day.checked_sub(day_borrowed);
                if days_held.is_none() {
                    return Err(LibraryError::InvalidReturnDay {
                        day_borrowed,
                        day_returned: day,
                    });
                };

                let fee = item.late_fee_cents(days_held.unwrap());

                (fee, member_id)
            }
            LoanStatus::Lost => return Err(LibraryError::ItemIsLost { id: item_id }),
            _ => return Err(LibraryError::ItemNotOnLoan { id: item_id }),
        };

        if let Some(item) = self
            .items
            .iter_mut()
            .find(|item| item.id == item_id)
        {
            item.status = LoanStatus::Available;
        }

        if let Some(member) = self
            .members
            .iter_mut()
            .find(|member| member.id == member_id)
        {
            member.borrowed_item_ids.retain(|id| id != &item_id);
        }

        Ok(fee)
    }
}
