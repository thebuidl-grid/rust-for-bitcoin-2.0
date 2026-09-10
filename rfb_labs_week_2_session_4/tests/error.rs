use rfb_labs_week_2_session_4::LibraryError;

#[test]
fn display_empty_title() {
    let err = LibraryError::EmptyTitle;
    let msg = err.to_string();
    assert!(msg.to_lowercase().contains("title"));
    assert!(!msg.is_empty());
}

#[test]
fn display_duplicate_item_id_includes_id() {
    let id = 10u32;
    let err = LibraryError::DuplicateItemId { id };
    let msg = err.to_string();
    assert!(msg.contains("item id"));
    assert!(msg.contains(&id.to_string()));
}

#[test]
fn display_duplicate_member_id_includes_id() {
    let id = 20u32;
    let err = LibraryError::DuplicateMemberId { id };
    let msg = err.to_string();
    assert!(msg.contains("member id"));
    assert!(msg.contains(&id.to_string()));
}

#[test]
fn display_item_not_found_includes_id() {
    let id = 30u32;
    let err = LibraryError::ItemNotFound { id };
    let msg = err.to_string();
    assert!(msg.contains("item id"));
    assert!(msg.contains(&id.to_string()));
    assert!(msg.contains("not found"));
}

#[test]
fn display_member_not_found_includes_id() {
    let id = 40u32;
    let err = LibraryError::MemberNotFound { id };
    let msg = err.to_string();
    assert!(msg.contains("member id"));
    assert!(msg.contains(&id.to_string()));
    assert!(msg.contains("not found"));
}

#[test]
fn display_item_already_on_loan_includes_both_ids() {
    let id = 50u32;
    let member_id = 60u32;
    let err = LibraryError::ItemAlreadyOnLoan { id, member_id };
    let msg = err.to_string();
    assert!(msg.contains("item id"));
    assert!(msg.contains("already on loan"));
    assert!(msg.contains(&id.to_string()));
    assert!(msg.contains(&member_id.to_string()));
}

#[test]
fn display_item_not_on_loan_includes_id() {
    let id = 70u32;
    let err = LibraryError::ItemNotOnLoan { id };
    let msg = err.to_string();
    assert!(msg.contains("item id"));
    assert!(msg.contains(&id.to_string()));
    assert!(msg.contains("not currently on loan") || msg.contains("not currently"));
}

#[test]
fn display_item_is_lost_includes_id() {
    let id = 80u32;
    let err = LibraryError::ItemIsLost { id };
    let msg = err.to_string();
    assert!(msg.contains("lost"));
    assert!(msg.contains(&id.to_string()));
}

#[test]
fn display_borrow_limit_reached_includes_member_id_and_limit() {
    let member_id = 90u32;
    let limit = 3usize;
    let err = LibraryError::BorrowLimitReached { member_id, limit };
    let msg = err.to_string();

    assert!(msg.contains("borrow limit"));
    assert!(msg.contains(&member_id.to_string()));
    assert!(msg.contains(&limit.to_string()));
}

#[test]
fn display_invalid_return_day_includes_both_days() {
    let day_borrowed = 1u32;
    let day_returned = 2u32;
    let err = LibraryError::InvalidReturnDay {
        day_borrowed,
        day_returned,
    };
    let msg = err.to_string();

    assert!(msg.contains("invalid return day"));
    assert!(msg.contains(&day_borrowed.to_string()));
    assert!(msg.contains(&day_returned.to_string()));
}
