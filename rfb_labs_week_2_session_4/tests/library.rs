use rfb_labs_week_2_session_4::LibraryError::ItemAlreadyOnLoan;
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
fn item_cannot_be_lent_twice() {
    let mut library = library_with_items();

    library.checkout(1, 100, 5).unwrap();

    assert_eq!(
        library.checkout(1, 100, 5),
        Err(ItemAlreadyOnLoan {
            id: 1,
            member_id: 100,
        })
    );
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
fn returning_a_book_on_time_does_not_incur_late_charges() {
    let mut library = library_with_items();

    // A book may be kept 21 days. Held for 30, so 9 days are overdue.
    library.checkout(1, 100, 10).unwrap();

    assert_eq!(library.return_item(1, 31), Ok(0));
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
fn returning_an_ebook_on_time_does_not_incur_late_charges() {
    let mut library = library_with_items();

    // A book may be kept 21 days. Held for 30, so 9 days are overdue.
    library.checkout(4, 100, 10).unwrap();

    assert_eq!(library.return_item(4, 40), Ok(0));
    assert_eq!(library.find_item(4).unwrap().status, LoanStatus::Available);
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
fn searching_a_borrowed_book_by_author_() {
    let mut library = library_with_items();

    library.checkout(1, 100, 10).unwrap();

    let found = library.items_by_author("Frank Herbert");

    assert_eq!(found.len(), 2);
    assert_eq!(found[0].title, "Dune");
    assert_eq!(
        found[0].status,
        LoanStatus::OnLoan {
            member_id: 100,
            day_borrowed: 10
        }
    );
}

#[test]
fn adding_item_with_empty_title_raises_empty_title() {
    let mut library = library_with_items();
    let (id, title, author, kind) = (3, "", "Frank Herbert", MediaKind::Book { pages: 320 });

    assert_eq!(
        library.add_item(Item::new(id, title.into(), author.into(), kind)),
        Err(LibraryError::EmptyTitle)
    )
}

#[test]
fn adding_item_twice_raises_duplicate_item_error() {
    let mut library = library_with_items();
    let (id, title, author, kind) = (1, "Dune", "Frank Herbert", MediaKind::Book { pages: 320 });

    assert_eq!(
        library.add_item(Item::new(id, title.into(), author.into(), kind)),
        Err(LibraryError::DuplicateItemId { id: 1 })
    )
}

#[test]
fn registering_a_member_twice_raises_duplicate_member_error() {
    let mut library = library_with_items();

    assert_eq!(
        library.register_member(Member::new(100, "Ada".into())),
        Err(LibraryError::DuplicateMemberId { id: 100 })
    )
}

#[test]
fn test_library_errors_individual_cases() {
    let mut library = library_with_items();

    assert_eq!(
        library.checkout(9999, 100, 10),
        Err(LibraryError::ItemNotFound { id: 9999 })
    );
    assert_eq!(
        library.return_item(9999, 100),
        Err(LibraryError::ItemNotFound { id: 9999 })
    );
    assert_eq!(
        library.checkout(1, 1, 10),
        Err(LibraryError::MemberNotFound { id: 1 })
    );

    library.checkout(1, 100, 0).unwrap();
    library.checkout(2, 100, 0).unwrap();
    library.checkout(3, 100, 0).unwrap();

    assert_eq!(
        library.checkout(4, 100, 0),
        Err(LibraryError::BorrowLimitReached {
            member_id: 100,
            limit: 3
        })
    );
}

#[test]
fn checkout_and_return_reject_a_lost_item() {
    let mut library = library_with_items();
    let mut lost = Item::new(
        5,
        "Lost Book".into(),
        "Ghost Writer".into(),
        MediaKind::Book { pages: 1 },
    );
    lost.status = LoanStatus::Lost;
    library.add_item(lost).unwrap();

    assert_eq!(
        library.checkout(5, 100, 3),
        Err(LibraryError::ItemIsLost { id: 5 })
    );
    assert_eq!(
        library.return_item(5, 3),
        Err(LibraryError::ItemIsLost { id: 5 })
    );
}

#[test]
fn return_item_rejects_an_item_not_on_loan() {
    let mut library = library_with_items();

    assert_eq!(
        library.return_item(1, 10),
        Err(LibraryError::ItemNotOnLoan { id: 1 })
    );
}

#[test]
fn return_before_the_borrow_day_is_invalid() {
    let mut library = library_with_items();

    library.checkout(1, 100, 10).unwrap();

    assert_eq!(
        library.return_item(1, 9),
        Err(LibraryError::InvalidReturnDay {
            day_borrowed: 10,
            day_returned: 9
        })
    );
}
