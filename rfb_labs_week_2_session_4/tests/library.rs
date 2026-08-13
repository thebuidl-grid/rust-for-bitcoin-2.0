use rfb_labs_week_2_session_4::{Item, Library, LibraryError, LoanStatus, MediaKind, Member};

// === Helpers

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

// === Starter tests (ignore removed)

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

// === Return tests

#[test]
fn on_time_return_owes_nothing() {
    let mut library = library_with_items();

    // Book: 21-day limit. Hold for exactly 21 days.
    library.checkout(1, 100, 0).unwrap();
    assert_eq!(library.return_item(1, 21), Ok(0));
}

#[test]
fn ebook_returned_late_still_owes_nothing() {
    let mut library = library_with_items();

    // Ebook (id 4): daily fee is 0 cents regardless of how late.
    library.checkout(4, 100, 0).unwrap();
    assert_eq!(library.return_item(4, 100), Ok(0));
}

#[test]
fn item_cannot_be_lent_twice() {
    let mut library = library_with_items();

    library
        .register_member(Member::new(101, "Bob".into()))
        .unwrap();

    library.checkout(1, 100, 0).unwrap();

    assert_eq!(
        library.checkout(1, 101, 0),
        Err(LibraryError::ItemAlreadyOnLoan {
            id: 1,
            member_id: 100,
        })
    );
}

// === Checkout validation error tests

#[test]
fn checkout_unknown_item_returns_item_not_found() {
    let mut library = library_with_items();
    assert_eq!(
        library.checkout(99, 100, 0),
        Err(LibraryError::ItemNotFound { id: 99 })
    );
}

#[test]
fn checkout_unknown_member_returns_member_not_found() {
    let mut library = library_with_items();
    assert_eq!(
        library.checkout(1, 999, 0),
        Err(LibraryError::MemberNotFound { id: 999 })
    );
}

#[test]
fn checkout_lost_item_returns_item_is_lost() {
    let mut library = Library::new();
    let mut item = Item::new(
        10,
        "Lost Book".into(),
        "Someone".into(),
        MediaKind::Book { pages: 100 },
    );
    item.status = LoanStatus::Lost;
    library.add_item(item).unwrap();
    library
        .register_member(Member::new(100, "Ada".into()))
        .unwrap();

    assert_eq!(
        library.checkout(10, 100, 0),
        Err(LibraryError::ItemIsLost { id: 10 })
    );
}

// === Return validation error tests

#[test]
fn return_unknown_item_returns_item_not_found() {
    let mut library = library_with_items();
    assert_eq!(
        library.return_item(99, 5),
        Err(LibraryError::ItemNotFound { id: 99 })
    );
}

#[test]
fn return_lost_item_returns_item_is_lost() {
    let mut library = Library::new();
    let mut item = Item::new(
        10,
        "Lost Book".into(),
        "Someone".into(),
        MediaKind::Book { pages: 100 },
    );
    item.status = LoanStatus::Lost;
    library.add_item(item).unwrap();

    assert_eq!(
        library.return_item(10, 5),
        Err(LibraryError::ItemIsLost { id: 10 })
    );
}

#[test]
fn return_available_item_returns_item_not_on_loan() {
    let mut library = library_with_items();
    assert_eq!(
        library.return_item(1, 5),
        Err(LibraryError::ItemNotOnLoan { id: 1 })
    );
}

#[test]
fn return_day_earlier_than_borrow_day_returns_invalid_return_day() {
    let mut library = library_with_items();
    library.checkout(1, 100, 10).unwrap();

    assert_eq!(
        library.return_item(1, 5),
        Err(LibraryError::InvalidReturnDay {
            day_borrowed: 10,
            day_returned: 5,
        })
    );
}

// === add_item / register_member validation tests

#[test]
fn add_item_rejects_empty_title() {
    let mut library = Library::new();
    let item = Item::new(1, "".into(), "Author".into(), MediaKind::Book { pages: 100 });
    assert_eq!(library.add_item(item), Err(LibraryError::EmptyTitle));
}

#[test]
fn add_item_rejects_duplicate_id() {
    let mut library = Library::new();
    library
        .add_item(Item::new(
            1,
            "First".into(),
            "Author".into(),
            MediaKind::Book { pages: 100 },
        ))
        .unwrap();

    let result = library.add_item(Item::new(
        1,
        "Second".into(),
        "Author".into(),
        MediaKind::Book { pages: 200 },
    ));
    assert_eq!(result, Err(LibraryError::DuplicateItemId { id: 1 }));
}

#[test]
fn register_member_rejects_duplicate_id() {
    let mut library = Library::new();
    library
        .register_member(Member::new(100, "Ada".into()))
        .unwrap();

    assert_eq!(
        library.register_member(Member::new(100, "Ada Clone".into())),
        Err(LibraryError::DuplicateMemberId { id: 100 })
    );
}

// === Author search includes borrowed items

#[test]
fn author_search_returns_borrowed_items_too() {
    let mut library = library_with_items();

    // Check out one of Frank Herbert's books — it should still appear in the search.
    library.checkout(1, 100, 0).unwrap();

    let found = library.items_by_author("Frank Herbert");
    assert_eq!(found.len(), 2);
}

// === available_items

#[test]
fn available_items_excludes_checked_out_items() {
    let mut library = library_with_items();
    let total = library.available_items().len();

    library.checkout(1, 100, 0).unwrap();

    assert_eq!(library.available_items().len(), total - 1);
}

// === longest_loan_item

#[test]
fn longest_loan_item_returns_book_over_audiobook_over_ebook() {
    let library = library_with_items();
    // Books have a 21-day limit — the longest of the three kinds stocked.
    let longest = library.longest_loan_item().unwrap();
    assert!(matches!(longest.kind, MediaKind::Book { .. }));
}
