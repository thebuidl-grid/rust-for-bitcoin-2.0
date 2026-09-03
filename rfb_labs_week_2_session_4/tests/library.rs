use rfb_labs_week_2_session_4::{Item, Library, LibraryError, LoanStatus, MediaKind, Member};

fn library_with_items() -> Library {
    let mut library = Library::new();

    for (id, title, author, kind) in [
        (1, "Dune", "Frank Herbert", MediaKind::Book { pages: 320 }),
        (
            2,
            "Children of Dune",
            "Frank Herbert",
            MediaKind::Book { pages: 180 },
        ),
        (
            3,
            "Project Hail Mary",
            "Andy Weir",
            MediaKind::Audiobook { minutes: 540 },
        ),
        (
            4,
            "The Rust Programming Language",
            "Steve Klabnik",
            MediaKind::Ebook { size_kb: 1_200 },
        ),
    ] {
        library
            .add_item(Item::new(id, title.into(), author.into(), kind))
            .unwrap();
    }

    library
        .register_member(Member::new(100, "Ada".into()))
        .unwrap();

    library
}

// These tests are ignored so the starter repository builds before students
// implement the TODOs. Remove `#[ignore]` from one test at a time while working.

#[test]
fn checkout_updates_both_the_item_and_the_member() {
    let mut library = library_with_items();

    library.checkout(1, 100, 5).unwrap();

    assert_eq!(
        library.find_item(1).unwrap().status,
        LoanStatus::OnLoan {
            member_id: 100,
            day_borrowed: 5,
        }
    );
    assert_eq!(library.find_member(100).unwrap().borrowed_item_ids, vec![1]);
}

#[test]
fn a_member_cannot_exceed_the_borrow_limit() {
    let mut library = library_with_items();

    library.checkout(1, 100, 0).unwrap();
    library.checkout(2, 100, 0).unwrap();
    library.checkout(3, 100, 0).unwrap();

    assert_eq!(
        library.checkout(4, 100, 0),
        Err(LibraryError::BorrowLimitReached {
            member_id: 100,
            limit: 3,
        })
    );
}

#[test]
fn returning_a_book_late_charges_a_daily_fee() {
    let mut library = library_with_items();

    // A book may be kept 21 days. Held for 30, so 9 days are overdue.
    library.checkout(1, 100, 10).unwrap();

    assert_eq!(library.return_item(1, 40), Ok(9 * 25));
    assert_eq!(library.find_item(1).unwrap().status, LoanStatus::Available);
    assert!(
        library
            .find_member(100)
            .unwrap()
            .borrowed_item_ids
            .is_empty()
    );
}

#[test]
fn searching_by_author_borrows_rather_than_clones() {
    let library = library_with_items();

    let found = library.items_by_author("Frank Herbert");

    assert_eq!(found.len(), 2);
    assert_eq!(found[0].title, "Dune");
    // `found` holds references into `library`, so these are the same item.
    assert!(std::ptr::eq(found[0], library.find_item(1).unwrap()));
}

#[test]
fn returning_a_book_on_time_charges_no_fee() {
    let mut library = library_with_items();

    // Book has 21 day loan period, return exactly on time
    library.checkout(1, 100, 10).unwrap();

    assert_eq!(library.return_item(1, 31), Ok(0)); // Day 10 + 21 days = day 31
    assert_eq!(library.find_item(1).unwrap().status, LoanStatus::Available);
}

#[test]
fn returning_an_ebook_late_never_charges_a_fee() {
    let mut library = library_with_items();

    // Ebook loan period is 7 days, but we hold it for 20 days
    library.checkout(4, 100, 5).unwrap();

    // 13 days late, but ebooks never charge fees
    assert_eq!(library.return_item(4, 25), Ok(0));
    assert_eq!(library.find_item(4).unwrap().status, LoanStatus::Available);
}

#[test]
fn error_empty_title() {
    let mut library = Library::new();

    let result = library.add_item(Item::new(
        1,
        "".to_string(),
        "Author".to_string(),
        MediaKind::Book { pages: 100 },
    ));

    assert_eq!(result, Err(LibraryError::EmptyTitle));
}

#[test]
fn error_duplicate_item_id() {
    let mut library = Library::new();

    library
        .add_item(Item::new(
            1,
            "First Book".to_string(),
            "Author".to_string(),
            MediaKind::Book { pages: 100 },
        ))
        .unwrap();

    let result = library.add_item(Item::new(
        1,
        "Second Book".to_string(),
        "Author".to_string(),
        MediaKind::Book { pages: 200 },
    ));

    assert_eq!(result, Err(LibraryError::DuplicateItemId { id: 1 }));
}

#[test]
fn error_duplicate_member_id() {
    let mut library = Library::new();

    library
        .register_member(Member::new(100, "Alice".to_string()))
        .unwrap();

    let result = library.register_member(Member::new(100, "Bob".to_string()));

    assert_eq!(result, Err(LibraryError::DuplicateMemberId { id: 100 }));
}

#[test]
fn error_item_not_found_on_checkout() {
    let mut library = library_with_items();

    let result = library.checkout(999, 100, 0);

    assert_eq!(result, Err(LibraryError::ItemNotFound { id: 999 }));
}

#[test]
fn error_member_not_found_on_checkout() {
    let mut library = library_with_items();

    let result = library.checkout(1, 999, 0);

    assert_eq!(result, Err(LibraryError::MemberNotFound { id: 999 }));
}

#[test]
fn error_item_is_lost() {
    let mut library = Library::new();

    let mut item = Item::new(
        1,
        "Lost Book".to_string(),
        "Author".to_string(),
        MediaKind::Book { pages: 100 },
    );
    item.status = LoanStatus::Lost;
    library.add_item(item).unwrap();
    library
        .register_member(Member::new(100, "Alice".to_string()))
        .unwrap();

    let result = library.checkout(1, 100, 0);

    assert_eq!(result, Err(LibraryError::ItemIsLost { id: 1 }));
}

#[test]
fn error_item_already_on_loan() {
    let mut library = library_with_items();

    library.checkout(1, 100, 0).unwrap();

    let result = library.checkout(1, 100, 1);

    assert_eq!(
        result,
        Err(LibraryError::ItemAlreadyOnLoan {
            id: 1,
            member_id: 100
        })
    );
}

#[test]
fn error_item_not_on_loan_when_returning() {
    let mut library = library_with_items();

    let result = library.return_item(1, 10);

    assert_eq!(result, Err(LibraryError::ItemNotOnLoan { id: 1 }));
}

#[test]
fn error_invalid_return_day() {
    let mut library = library_with_items();

    library.checkout(1, 100, 20).unwrap();

    // Try to return before the borrow day
    let result = library.return_item(1, 15);

    assert_eq!(
        result,
        Err(LibraryError::InvalidReturnDay {
            day_borrowed: 20,
            day_returned: 15
        })
    );
}

#[test]
fn error_item_not_found_on_return() {
    let mut library = library_with_items();

    let result = library.return_item(999, 10);

    assert_eq!(result, Err(LibraryError::ItemNotFound { id: 999 }));
}

#[test]
fn error_lost_item_cannot_be_returned() {
    let mut library = Library::new();

    let mut item = Item::new(
        1,
        "Lost Book".to_string(),
        "Author".to_string(),
        MediaKind::Book { pages: 100 },
    );
    item.status = LoanStatus::Lost;
    library.add_item(item).unwrap();

    let result = library.return_item(1, 10);

    assert_eq!(result, Err(LibraryError::ItemIsLost { id: 1 }));
}
