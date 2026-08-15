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
fn the_same_item_cannot_be_lent_twice() {
    let mut library = library_with_items();

    library.checkout(1, 100, 0).unwrap();

    assert_eq!(
        library.checkout(1, 100, 1),
        Err(LibraryError::ItemAlreadyOnLoan {
            id: 1,
            member_id: 100,
        })
    );
}

#[test]
fn returning_on_time_owes_nothing() {
    let mut library = library_with_items();

    library.checkout(1, 100, 10).unwrap();

    assert_eq!(library.return_item(1, 31), Ok(0));
}

#[test]
fn returning_an_ebook_late_still_owes_nothing() {
    let mut library = library_with_items();

    library.checkout(4, 100, 10).unwrap();

    assert_eq!(library.return_item(4, 100), Ok(0));
}

#[test]
fn available_items_only_returns_available_items() {
    let mut library = library_with_items();

    library.checkout(1, 100, 0).unwrap();

    let available_ids: Vec<u32> = library
        .available_items()
        .iter()
        .map(|item| item.id)
        .collect();
    assert_eq!(available_ids, vec![2, 3, 4]);
}

#[test]
fn longest_loan_item_uses_loan_terms() {
    let library = library_with_items();

    assert_eq!(library.longest_loan_item().unwrap().id, 1);
}

#[test]
fn add_item_rejects_empty_titles() {
    let mut library = Library::new();

    assert_eq!(
        library.add_item(Item::new(
            1,
            "   ".into(),
            "Nobody".into(),
            MediaKind::Book { pages: 10 },
        )),
        Err(LibraryError::EmptyTitle)
    );
}

#[test]
fn add_item_rejects_duplicate_ids() {
    let mut library = library_with_items();

    assert_eq!(
        library.add_item(Item::new(
            1,
            "Duplicate".into(),
            "Nobody".into(),
            MediaKind::Book { pages: 10 },
        )),
        Err(LibraryError::DuplicateItemId { id: 1 })
    );
}

#[test]
fn register_member_rejects_duplicate_ids() {
    let mut library = library_with_items();

    assert_eq!(
        library.register_member(Member::new(100, "Grace".into())),
        Err(LibraryError::DuplicateMemberId { id: 100 })
    );
}

#[test]
fn checkout_reports_unknown_item_first() {
    let mut library = library_with_items();

    assert_eq!(
        library.checkout(99, 999, 0),
        Err(LibraryError::ItemNotFound { id: 99 })
    );
}

#[test]
fn checkout_reports_unknown_member_after_item_exists() {
    let mut library = library_with_items();

    assert_eq!(
        library.checkout(1, 999, 0),
        Err(LibraryError::MemberNotFound { id: 999 })
    );
}

#[test]
fn checkout_rejects_lost_items() {
    let mut library = library_with_items();
    let mut lost_item = Item::new(
        9,
        "Missing Manual".into(),
        "Archivist".into(),
        MediaKind::Book { pages: 100 },
    );
    lost_item.status = LoanStatus::Lost;
    library.add_item(lost_item).unwrap();

    assert_eq!(
        library.checkout(9, 100, 0),
        Err(LibraryError::ItemIsLost { id: 9 })
    );
}

#[test]
fn return_reports_unknown_item() {
    let mut library = library_with_items();

    assert_eq!(
        library.return_item(99, 0),
        Err(LibraryError::ItemNotFound { id: 99 })
    );
}

#[test]
fn return_rejects_lost_items() {
    let mut library = library_with_items();
    let mut lost_item = Item::new(
        9,
        "Missing Manual".into(),
        "Archivist".into(),
        MediaKind::Book { pages: 100 },
    );
    lost_item.status = LoanStatus::Lost;
    library.add_item(lost_item).unwrap();

    assert_eq!(
        library.return_item(9, 0),
        Err(LibraryError::ItemIsLost { id: 9 })
    );
}

#[test]
fn return_rejects_items_not_on_loan() {
    let mut library = library_with_items();

    assert_eq!(
        library.return_item(1, 0),
        Err(LibraryError::ItemNotOnLoan { id: 1 })
    );
}

#[test]
fn return_rejects_days_before_the_borrow_day() {
    let mut library = library_with_items();

    library.checkout(1, 100, 10).unwrap();

    assert_eq!(
        library.return_item(1, 9),
        Err(LibraryError::InvalidReturnDay {
            day_borrowed: 10,
            day_returned: 9,
        })
    );
}
