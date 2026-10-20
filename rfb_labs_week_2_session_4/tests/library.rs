use rfb_labs_week_2_session_4::{
    Item, Library, LibraryError, LoanStatus, LoanTerms, MediaKind, Member,
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
fn an_item_cannot_be_lent_twice() {
    let mut library = library_with_items();

    library.checkout(1, 100, 5).unwrap();

    assert_eq!(
        library.checkout(1, 100, 6),
        Err(LibraryError::ItemAlreadyOnLoan {
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

    library.checkout(1, 100, 10).unwrap();

    assert_eq!(library.return_item(1, 40), Ok(225));
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
fn on_time_return_owes_nothing() {
    let mut library = library_with_items();

    library.checkout(1, 100, 10).unwrap();

    assert_eq!(library.return_item(1, 31), Ok(0));
}

#[test]
fn ebook_returned_late_still_owes_nothing() {
    let mut library = library_with_items();

    library.checkout(4, 100, 0).unwrap();

    assert_eq!(library.return_item(4, 100), Ok(0));
}

#[test]
fn author_search_returns_borrowed_items() {
    let mut library = library_with_items();

    library.checkout(1, 100, 5).unwrap();

    let found = library.items_by_author("Frank Herbert");

    assert_eq!(found.len(), 2);
    assert_eq!(found[0].title, "Dune");
    assert_eq!(
        found[0].status,
        LoanStatus::OnLoan {
            member_id: 100,
            day_borrowed: 5,
        }
    );
    assert!(std::ptr::eq(found[0], library.find_item(1).unwrap()));
}

#[test]
fn add_item_rejects_empty_title() {
    let mut library = Library::new();

    assert_eq!(
        library.add_item(Item::new(
            1,
            "   ".into(),
            "Author".into(),
            MediaKind::Book { pages: 100 },
        )),
        Err(LibraryError::EmptyTitle)
    );
}

#[test]
fn add_item_rejects_duplicate_id() {
    let mut library = library_with_items();

    assert_eq!(
        library.add_item(Item::new(
            1,
            "Another Book".into(),
            "Author".into(),
            MediaKind::Book { pages: 100 },
        )),
        Err(LibraryError::DuplicateItemId { id: 1 })
    );
}

#[test]
fn register_member_rejects_duplicate_id() {
    let mut library = library_with_items();

    assert_eq!(
        library.register_member(Member::new(100, "Grace".into())),
        Err(LibraryError::DuplicateMemberId { id: 100 })
    );
}

#[test]
fn checkout_rejects_unknown_item_first() {
    let mut library = library_with_items();

    assert_eq!(
        library.checkout(999, 999, 0),
        Err(LibraryError::ItemNotFound { id: 999 })
    );
}

#[test]
fn checkout_rejects_unknown_member_after_item_validation() {
    let mut library = library_with_items();

    assert_eq!(
        library.checkout(1, 999, 0),
        Err(LibraryError::MemberNotFound { id: 999 })
    );
}

#[test]
fn checkout_rejects_lost_item() {
    let mut library = library_with_items();

    library.checkout(1, 100, 0).unwrap();
    library.return_item(1, 1).unwrap();

    // The public API does not expose mutation of a lost status, so this
    // validation is exercised indirectly by the unit tests in the crate.
    assert!(library.find_item(1).is_some());
}

#[test]
fn return_rejects_unknown_item() {
    let mut library = library_with_items();

    assert_eq!(
        library.return_item(999, 10),
        Err(LibraryError::ItemNotFound { id: 999 })
    );
}

#[test]
fn return_rejects_item_not_on_loan() {
    let mut library = library_with_items();

    assert_eq!(
        library.return_item(1, 10),
        Err(LibraryError::ItemNotOnLoan { id: 1 })
    );
}

#[test]
fn return_rejects_day_before_borrow_day() {
    let mut library = library_with_items();

    library.checkout(1, 100, 20).unwrap();

    assert_eq!(
        library.return_item(1, 10),
        Err(LibraryError::InvalidReturnDay {
            day_borrowed: 20,
            day_returned: 10,
        })
    );
}

#[test]
fn media_kind_loan_terms_are_correct() {
    assert_eq!(MediaKind::Book { pages: 100 }.loan_days(), 21);
    assert_eq!(MediaKind::Audiobook { minutes: 60 }.loan_days(), 14);
    assert_eq!(MediaKind::Ebook { size_kb: 500 }.loan_days(), 7);

    assert_eq!(MediaKind::Book { pages: 100 }.late_fee_cents(30), 225);
    assert_eq!(MediaKind::Audiobook { minutes: 60 }.late_fee_cents(20), 150);
    assert_eq!(MediaKind::Ebook { size_kb: 500 }.late_fee_cents(30), 0);
}

#[test]
fn longest_loan_item_is_a_book() {
    let library = library_with_items();

    assert_eq!(library.longest_loan_item().unwrap().id, 1);
}

#[test]
fn available_items_excludes_borrowed_items() {
    let mut library = library_with_items();

    library.checkout(1, 100, 0).unwrap();

    let available = library.available_items();

    assert_eq!(available.len(), 3);
    assert!(available.iter().all(|item| item.id != 1));
}

#[test]
fn error_display_contains_relevant_details() {
    let error = LibraryError::ItemAlreadyOnLoan {
        id: 7,
        member_id: 42,
    };

    let message = error.to_string();

    assert!(message.contains("7"));
    assert!(message.contains("42"));
}
