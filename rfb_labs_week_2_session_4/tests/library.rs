use rfb_labs_week_2_session_4::{
    Item, Library, LibraryError, LoanStatus, LoanTerms, MAX_ITEMS_PER_MEMBER, MediaKind, Member,
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

/// Nothing in the public API marks an item lost, but `Item`'s fields are
/// public, so a lost item can be stocked directly.
fn library_with_a_lost_item() -> Library {
    let mut library = library_with_items();

    let mut lost = Item::new(
        9,
        "Missing Atlas".into(),
        "Anon".into(),
        MediaKind::Book { pages: 90 },
    );
    lost.status = LoanStatus::Lost;
    library.add_item(lost).unwrap();

    library
}

// ---------------------------------------------------------------------------
// Checkout
// ---------------------------------------------------------------------------

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
    library
        .register_member(Member::new(101, "Grace".into()))
        .unwrap();

    library.checkout(1, 100, 0).unwrap();

    assert_eq!(
        library.checkout(1, 101, 1),
        Err(LibraryError::ItemAlreadyOnLoan {
            id: 1,
            member_id: 100,
        })
    );
    // The failed attempt left the second member holding nothing.
    assert!(
        library
            .find_member(101)
            .unwrap()
            .borrowed_item_ids
            .is_empty()
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
            limit: MAX_ITEMS_PER_MEMBER,
        })
    );
    // The rejected item is still on the shelf.
    assert_eq!(library.find_item(4).unwrap().status, LoanStatus::Available);
}

#[test]
fn a_returned_item_frees_a_slot_under_the_limit() {
    let mut library = library_with_items();

    library.checkout(1, 100, 0).unwrap();
    library.checkout(2, 100, 0).unwrap();
    library.checkout(3, 100, 0).unwrap();
    library.return_item(2, 1).unwrap();

    assert_eq!(library.checkout(4, 100, 1), Ok(()));
}

#[test]
fn checkout_reports_the_unknown_item_before_the_unknown_member() {
    let mut library = library_with_items();

    // Both ids are wrong; the documented order puts the item first.
    assert_eq!(
        library.checkout(999, 998, 0),
        Err(LibraryError::ItemNotFound { id: 999 })
    );
}

#[test]
fn checkout_rejects_an_unknown_member() {
    let mut library = library_with_items();

    assert_eq!(
        library.checkout(1, 998, 0),
        Err(LibraryError::MemberNotFound { id: 998 })
    );
    assert_eq!(library.find_item(1).unwrap().status, LoanStatus::Available);
}

#[test]
fn checkout_rejects_a_lost_item_before_anything_else() {
    let mut library = library_with_a_lost_item();

    assert_eq!(
        library.checkout(9, 100, 0),
        Err(LibraryError::ItemIsLost { id: 9 })
    );
}

// ---------------------------------------------------------------------------
// Returns and fees
// ---------------------------------------------------------------------------

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
fn returning_on_time_owes_nothing() {
    let mut library = library_with_items();

    library.checkout(1, 100, 10).unwrap();

    // Day 31 is exactly 21 days later — the last day of the loan.
    assert_eq!(library.return_item(1, 31), Ok(0));
}

#[test]
fn returning_the_same_day_owes_nothing() {
    let mut library = library_with_items();

    library.checkout(3, 100, 7).unwrap();

    assert_eq!(library.return_item(3, 7), Ok(0));
}

#[test]
fn an_ebook_returned_late_still_owes_nothing() {
    let mut library = library_with_items();

    // Item 4 is an ebook: a 7 day loan, held for 100 days.
    library.checkout(4, 100, 0).unwrap();

    assert_eq!(library.return_item(4, 100), Ok(0));
}

#[test]
fn a_late_audiobook_is_charged_from_day_fifteen() {
    let mut library = library_with_items();

    // 14 day loan, held 20 days, so 6 days overdue.
    library.checkout(3, 100, 0).unwrap();

    assert_eq!(library.return_item(3, 20), Ok(6 * 25));
}

#[test]
fn returning_an_unknown_item_is_an_error() {
    let mut library = library_with_items();

    assert_eq!(
        library.return_item(999, 0),
        Err(LibraryError::ItemNotFound { id: 999 })
    );
}

#[test]
fn returning_an_item_that_is_not_on_loan_is_an_error() {
    let mut library = library_with_items();

    assert_eq!(
        library.return_item(1, 0),
        Err(LibraryError::ItemNotOnLoan { id: 1 })
    );
}

#[test]
fn returning_a_lost_item_is_an_error() {
    let mut library = library_with_a_lost_item();

    assert_eq!(
        library.return_item(9, 0),
        Err(LibraryError::ItemIsLost { id: 9 })
    );
}

#[test]
fn returning_before_the_borrow_day_is_an_error() {
    let mut library = library_with_items();

    library.checkout(1, 100, 10).unwrap();

    assert_eq!(
        library.return_item(1, 9),
        Err(LibraryError::InvalidReturnDay {
            day_borrowed: 10,
            day_returned: 9,
        })
    );
    // The rejected return left the loan untouched.
    assert_eq!(
        library.find_item(1).unwrap().status,
        LoanStatus::OnLoan {
            member_id: 100,
            day_borrowed: 10,
        }
    );
    assert_eq!(library.find_member(100).unwrap().borrowed_item_ids, vec![1]);
}

// ---------------------------------------------------------------------------
// Stocking and registration
// ---------------------------------------------------------------------------

#[test]
fn an_item_needs_a_title() {
    let mut library = Library::new();

    assert_eq!(
        library.add_item(Item::new(
            1,
            String::new(),
            "Anon".into(),
            MediaKind::Book { pages: 10 },
        )),
        Err(LibraryError::EmptyTitle)
    );
    assert_eq!(
        library.add_item(Item::new(
            1,
            "   ".into(),
            "Anon".into(),
            MediaKind::Book { pages: 10 },
        )),
        Err(LibraryError::EmptyTitle)
    );
    assert!(library.find_item(1).is_none());
}

#[test]
fn item_ids_are_unique() {
    let mut library = library_with_items();

    assert_eq!(
        library.add_item(Item::new(
            1,
            "A Different Dune".into(),
            "Someone Else".into(),
            MediaKind::Book { pages: 10 },
        )),
        Err(LibraryError::DuplicateItemId { id: 1 })
    );
    // The original is untouched.
    assert_eq!(library.find_item(1).unwrap().title, "Dune");
}

#[test]
fn member_ids_are_unique() {
    let mut library = library_with_items();

    assert_eq!(
        library.register_member(Member::new(100, "Impostor".into())),
        Err(LibraryError::DuplicateMemberId { id: 100 })
    );
    assert_eq!(library.find_member(100).unwrap().name, "Ada");
}

// ---------------------------------------------------------------------------
// Borrowing lookups
// ---------------------------------------------------------------------------

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
fn author_search_includes_items_that_are_on_loan() {
    let mut library = library_with_items();

    library.checkout(1, 100, 0).unwrap();

    // Searching the catalogue is not the same as searching the shelf.
    let found = library.items_by_author("Frank Herbert");
    assert_eq!(found.len(), 2);
    assert!(found.iter().any(|item| item.id == 1));

    let available = library.available_items();
    assert!(!available.iter().any(|item| item.id == 1));
    assert_eq!(available.len(), 3);
}

#[test]
fn an_unknown_id_is_none_rather_than_a_panic() {
    let library = library_with_items();

    assert!(library.find_item(999).is_none());
    assert!(library.find_member(999).is_none());
    assert!(library.items_by_author("Nobody").is_empty());
}

#[test]
fn the_longest_loan_item_is_the_book() {
    let library = library_with_items();

    let longest = library.longest_loan_item().unwrap();

    assert_eq!(longest.loan_days(), 21);
    // Ties resolve to the first stocked item.
    assert_eq!(longest.id, 1);
    assert!(Library::new().longest_loan_item().is_none());
}

#[test]
fn filter_items_expresses_the_specific_lookups() {
    let library = library_with_items();

    let by_author = library.filter_items(|item| item.author == "Frank Herbert");
    assert_eq!(
        by_author.len(),
        library.items_by_author("Frank Herbert").len()
    );

    let available = library.filter_items(|item| item.status == LoanStatus::Available);
    assert_eq!(available.len(), library.available_items().len());

    let long_books = library.filter_items(|item| item.loan_days() >= 14);
    assert_eq!(long_books.len(), 3);
}

// ---------------------------------------------------------------------------
// Trait behaviour
// ---------------------------------------------------------------------------

#[test]
fn loan_terms_come_from_the_media_kind() {
    let book = MediaKind::Book { pages: 320 };
    let audiobook = MediaKind::Audiobook { minutes: 540 };
    let ebook = MediaKind::Ebook { size_kb: 1_200 };

    assert_eq!((book.loan_days(), book.daily_late_fee_cents()), (21, 25));
    assert_eq!(
        (audiobook.loan_days(), audiobook.daily_late_fee_cents()),
        (14, 25)
    );
    assert_eq!((ebook.loan_days(), ebook.daily_late_fee_cents()), (7, 0));

    // The item's terms are the kind's terms.
    let item = Item::new(1, "Dune".into(), "Frank Herbert".into(), book);
    assert_eq!(item.loan_days(), book.loan_days());
    assert_eq!(item.late_fee_cents(25), book.late_fee_cents(25));
}

#[test]
fn the_shared_fee_formula_saturates_instead_of_wrapping() {
    let book = MediaKind::Book { pages: 320 };

    assert_eq!(book.late_fee_cents(0), 0);
    assert_eq!(book.late_fee_cents(21), 0);
    assert_eq!(book.late_fee_cents(22), 25);
    assert_eq!(book.late_fee_cents(u32::MAX), u32::MAX);
}

#[test]
fn errors_name_the_ids_they_carry() {
    assert_eq!(
        LibraryError::ItemAlreadyOnLoan {
            id: 1,
            member_id: 100,
        }
        .to_string(),
        "item 1 is already on loan to member 100"
    );
    assert_eq!(
        LibraryError::BorrowLimitReached {
            member_id: 100,
            limit: 3,
        }
        .to_string(),
        "member 100 already holds the limit of 3 items"
    );
    assert_eq!(
        LibraryError::InvalidReturnDay {
            day_borrowed: 10,
            day_returned: 9,
        }
        .to_string(),
        "day 9 is before the borrow day 10"
    );

    // No variant may fall through to an empty or placeholder message.
    for error in [
        LibraryError::EmptyTitle,
        LibraryError::DuplicateItemId { id: 1 },
        LibraryError::DuplicateMemberId { id: 100 },
        LibraryError::ItemNotFound { id: 1 },
        LibraryError::MemberNotFound { id: 100 },
        LibraryError::ItemNotOnLoan { id: 1 },
        LibraryError::ItemIsLost { id: 1 },
    ] {
        let message = error.to_string();
        assert!(!message.is_empty(), "{error:?} has no message");
        assert!(!message.contains("todo"), "{error:?} left a placeholder");
    }
}

#[test]
fn display_describes_an_item_and_its_status() {
    let mut library = library_with_items();

    assert_eq!(
        library.find_item(1).unwrap().to_string(),
        "#1 \"Dune\" by Frank Herbert (book, 320 pages) — available"
    );

    library.checkout(1, 100, 5).unwrap();

    assert_eq!(
        library.find_item(1).unwrap().to_string(),
        "#1 \"Dune\" by Frank Herbert (book, 320 pages) — on loan to member 100 since day 5"
    );
    assert_eq!(
        MediaKind::Ebook { size_kb: 1_200 }.to_string(),
        "ebook, 1200 kB"
    );
    assert_eq!(LoanStatus::Lost.to_string(), "lost");
}
