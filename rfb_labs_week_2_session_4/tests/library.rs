use rfb_labs_week_2_session_4::{
    Item, Library, LibraryError, LoanStatus, MAX_ITEMS_PER_MEMBER, MediaKind, Member,
};

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
fn test_successful_checkout() {
    let mut lib = library_with_items();

    assert_eq!(lib.checkout(1, 100, 10), Ok(()));
    let item = lib.find_item(1).unwrap();
    assert_eq!(
        item.status,
        LoanStatus::OnLoan {
            member_id: 100,
            day_borrowed: 10
        }
    );
}

#[test]
fn test_item_cannot_be_lent_twice() {
    let mut lib = library_with_items();
    lib.checkout(1, 100, 10).unwrap();

    assert_eq!(
        lib.checkout(1, 100, 12),
        Err(LibraryError::ItemAlreadyOnLoan {
            id: 1,
            member_id: 100
        })
    );
}

#[test]
fn test_borrow_limit_reached() {
    let mut lib = library_with_items();

    // Borrow 3 items (the maximum allowed limit: items 1, 2, and 3)
    lib.checkout(1, 100, 1).unwrap();
    lib.checkout(2, 100, 1).unwrap();
    lib.checkout(3, 100, 1).unwrap();

    // 4th checkout should fail with BorrowLimitReached
    assert_eq!(
        lib.checkout(4, 100, 1),
        Err(LibraryError::BorrowLimitReached {
            member_id: 100,
            limit: MAX_ITEMS_PER_MEMBER
        })
    );
}

#[test]
fn test_late_return_fee() {
    let mut lib = library_with_items(); // Item 1 (Dune) has 21 loan days & 25c/day late fee

    lib.checkout(1, 100, 1).unwrap();
    // Borrowed day 1, due day 22. Returned day 27 -> 5 days late -> 5 * 25 = 125 cents
    let fee = lib.return_item(1, 27).unwrap();
    assert_eq!(fee, 125);
}

#[test]
fn test_ontime_return_owes_nothing() {
    let mut lib = library_with_items();

    lib.checkout(1, 100, 1).unwrap();
    // Returned on day 22 (21 days held -> exactly on time)
    let fee = lib.return_item(1, 22).unwrap();
    assert_eq!(fee, 0);
}

#[test]
fn test_ebook_returned_late_owes_nothing() {
    let mut lib = library_with_items(); // Item 4 is an Ebook

    lib.checkout(4, 100, 1).unwrap();
    // Ebooks have 7 loan days, but 0 daily late fee. Returned on day 100.
    let fee = lib.return_item(4, 100).unwrap();
    assert_eq!(fee, 0);
}

#[test]
fn test_author_search_includes_borrowed_items() {
    let mut lib = library_with_items();
    lib.checkout(1, 100, 1).unwrap();

    let author_items = lib.items_by_author("Frank Herbert");
    assert_eq!(author_items.len(), 2);
}

#[test]
fn test_validation_errors() {
    let mut lib = Library::new();

    // Empty title error
    let empty_item = Item::new(
        1,
        "   ".into(),
        "Author".into(),
        MediaKind::Book { pages: 100 },
    );
    assert_eq!(lib.add_item(empty_item), Err(LibraryError::EmptyTitle));

    // Duplicate item ID error
    let item1 = Item::new(
        1,
        "Book 1".into(),
        "Author".into(),
        MediaKind::Book { pages: 100 },
    );
    let item2 = Item::new(
        1,
        "Book 2".into(),
        "Author".into(),
        MediaKind::Book { pages: 100 },
    );
    lib.add_item(item1).unwrap();
    assert_eq!(
        lib.add_item(item2),
        Err(LibraryError::DuplicateItemId { id: 1 })
    );

    // Duplicate member ID error
    let m1 = Member::new(10, "Alice".into());
    let m2 = Member::new(10, "Bob".into());
    lib.register_member(m1).unwrap();
    assert_eq!(
        lib.register_member(m2),
        Err(LibraryError::DuplicateMemberId { id: 10 })
    );

    // Item not found on checkout
    assert_eq!(
        lib.checkout(99, 10, 1),
        Err(LibraryError::ItemNotFound { id: 99 })
    );

    // Member not found on checkout
    assert_eq!(
        lib.checkout(1, 99, 1),
        Err(LibraryError::MemberNotFound { id: 99 })
    );

    // Item not on loan when returned
    assert_eq!(
        lib.return_item(1, 10),
        Err(LibraryError::ItemNotOnLoan { id: 1 })
    );

    // Invalid return day (return date earlier than borrow date)
    lib.checkout(1, 10, 15).unwrap();
    assert_eq!(
        lib.return_item(1, 10),
        Err(LibraryError::InvalidReturnDay {
            day_borrowed: 15,
            day_returned: 10
        })
    );
}
