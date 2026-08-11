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

// ============================================================================
// Core Functional Tests
// ============================================================================

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

    // Trying to lend the same item to another member must fail
    assert_eq!(
        library.checkout(1, 101, 2),
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
            limit: MAX_ITEMS_PER_MEMBER,
        })
    );
}

#[test]
fn returning_a_book_late_charges_a_daily_fee() {
    let mut library = library_with_items();

    // A book may be kept 21 days. Held for 30 (day 10 to day 40), so 9 days overdue.
    // 9 days * 25 cents = 225 cents.
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
fn an_on_time_return_owes_nothing() {
    let mut library = library_with_items();

    // Book borrowed on day 5, returned on day 26 (exactly 21 days held)
    library.checkout(1, 100, 5).unwrap();
    assert_eq!(library.return_item(1, 26), Ok(0));
    assert_eq!(library.find_item(1).unwrap().status, LoanStatus::Available);
    assert!(
        library
            .find_member(100)
            .unwrap()
            .borrowed_item_ids
            .is_empty()
    );

    // Book borrowed on day 0, returned same day (0 days held)
    library.checkout(2, 100, 0).unwrap();
    assert_eq!(library.return_item(2, 0), Ok(0));
}

#[test]
fn an_audiobook_returned_late_charges_a_daily_fee() {
    let mut library = library_with_items();

    // Audiobook (item 3) may be kept 14 days. Borrowed on day 10, returned on day 30 (20 days held).
    // 20 - 14 = 6 days overdue * 25 cents = 150 cents.
    library.checkout(3, 100, 10).unwrap();
    assert_eq!(library.return_item(3, 30), Ok(6 * 25));
}

#[test]
fn an_ebook_returned_late_still_owes_nothing() {
    let mut library = library_with_items();

    // Ebook (item 4) has a 7-day loan window, but daily late fee is 0.
    // Borrowed on day 0, returned on day 100 (100 days held).
    library.checkout(4, 100, 0).unwrap();
    assert_eq!(library.return_item(4, 100), Ok(0));
    assert_eq!(library.find_item(4).unwrap().status, LoanStatus::Available);
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
fn author_search_returns_borrowed_and_available_items() {
    let mut library = library_with_items();

    // Check out one of Frank Herbert's books
    library.checkout(1, 100, 5).unwrap();

    let found = library.items_by_author("Frank Herbert");
    assert_eq!(found.len(), 2);
    assert!(
        found
            .iter()
            .any(|item| item.id == 1 && item.status != LoanStatus::Available)
    );
    assert!(
        found
            .iter()
            .any(|item| item.id == 2 && item.status == LoanStatus::Available)
    );
}

#[test]
fn available_items_filters_out_on_loan_and_lost_items() {
    let mut library = library_with_items();

    assert_eq!(library.available_items().len(), 4);

    library.checkout(1, 100, 0).unwrap();
    assert_eq!(library.available_items().len(), 3);
    assert!(!library.available_items().iter().any(|item| item.id == 1));
}

#[test]
fn longest_loan_item_finds_item_with_longest_term() {
    let library = library_with_items();

    // Books have 21 days, Audiobooks 14, Ebooks 7.
    let longest = library.longest_loan_item().unwrap();
    assert_eq!(longest.loan_days(), 21);

    let empty_library = Library::new();
    assert_eq!(empty_library.longest_loan_item(), None);
}

// ============================================================================
// Validation and Error Handling Tests
// ============================================================================

#[test]
fn add_item_rejects_empty_title() {
    let mut library = Library::new();

    let empty_item = Item::new(
        1,
        "".into(),
        "Author".into(),
        MediaKind::Book { pages: 100 },
    );
    assert_eq!(library.add_item(empty_item), Err(LibraryError::EmptyTitle));

    let whitespace_item = Item::new(
        2,
        "   ".into(),
        "Author".into(),
        MediaKind::Book { pages: 100 },
    );
    assert_eq!(
        library.add_item(whitespace_item),
        Err(LibraryError::EmptyTitle)
    );
}

#[test]
fn add_item_rejects_duplicate_id() {
    let mut library = Library::new();

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
        MediaKind::Book { pages: 200 },
    );

    assert_eq!(library.add_item(item1), Ok(()));
    assert_eq!(
        library.add_item(item2),
        Err(LibraryError::DuplicateItemId { id: 1 })
    );
}

#[test]
fn register_member_rejects_duplicate_id() {
    let mut library = Library::new();

    let m1 = Member::new(100, "Alice".into());
    let m2 = Member::new(100, "Bob".into());

    assert_eq!(library.register_member(m1), Ok(()));
    assert_eq!(
        library.register_member(m2),
        Err(LibraryError::DuplicateMemberId { id: 100 })
    );
}

#[test]
fn checkout_validation_order_and_cases() {
    let mut library = library_with_items();

    // 1. Unknown item
    assert_eq!(
        library.checkout(999, 100, 0),
        Err(LibraryError::ItemNotFound { id: 999 })
    );

    // 2. Unknown member
    assert_eq!(
        library.checkout(1, 999, 0),
        Err(LibraryError::MemberNotFound { id: 999 })
    );

    // 3. Lost item
    let mut lost_item = Item::new(
        5,
        "Lost Book".into(),
        "Unknown".into(),
        MediaKind::Book { pages: 50 },
    );
    lost_item.status = LoanStatus::Lost;
    library.add_item(lost_item).unwrap();

    assert_eq!(
        library.checkout(5, 100, 0),
        Err(LibraryError::ItemIsLost { id: 5 })
    );

    // 4. Already on loan
    library.checkout(1, 100, 0).unwrap();
    library
        .register_member(Member::new(101, "Bob".into()))
        .unwrap();
    assert_eq!(
        library.checkout(1, 101, 0),
        Err(LibraryError::ItemAlreadyOnLoan {
            id: 1,
            member_id: 100,
        })
    );
}

#[test]
fn return_item_validation_order_and_cases() {
    let mut library = library_with_items();

    // 1. Unknown item
    assert_eq!(
        library.return_item(999, 10),
        Err(LibraryError::ItemNotFound { id: 999 })
    );

    // 2. Lost item
    let mut lost_item = Item::new(
        5,
        "Lost Book".into(),
        "Unknown".into(),
        MediaKind::Book { pages: 50 },
    );
    lost_item.status = LoanStatus::Lost;
    library.add_item(lost_item).unwrap();

    assert_eq!(
        library.return_item(5, 10),
        Err(LibraryError::ItemIsLost { id: 5 })
    );

    // 3. Item not on loan
    assert_eq!(
        library.return_item(1, 10),
        Err(LibraryError::ItemNotOnLoan { id: 1 })
    );

    // 4. Invalid return day (day_returned < day_borrowed)
    library.checkout(1, 100, 15).unwrap();
    assert_eq!(
        library.return_item(1, 10),
        Err(LibraryError::InvalidReturnDay {
            day_borrowed: 15,
            day_returned: 10,
        })
    );
}

#[test]
fn display_implementations_produce_human_readable_text() {
    let book = MediaKind::Book { pages: 300 };
    let audiobook = MediaKind::Audiobook { minutes: 480 };
    let ebook = MediaKind::Ebook { size_kb: 2048 };

    assert_eq!(book.to_string(), "Book (300 pages)");
    assert_eq!(audiobook.to_string(), "Audiobook (480 mins)");
    assert_eq!(ebook.to_string(), "Ebook (2048 KB)");

    assert_eq!(LoanStatus::Available.to_string(), "Available");
    assert_eq!(
        LoanStatus::OnLoan {
            member_id: 100,
            day_borrowed: 5
        }
        .to_string(),
        "On loan to member 100 (borrowed day 5)"
    );
    assert_eq!(LoanStatus::Lost.to_string(), "Lost");

    let item = Item::new(1, "Rust Book".into(), "Steve".into(), ebook);
    assert_eq!(
        item.to_string(),
        "\"Rust Book\" by Steve [Ebook (2048 KB)] - Available"
    );

    let err1 = LibraryError::EmptyTitle;
    let err2 = LibraryError::DuplicateItemId { id: 42 };
    let err3 = LibraryError::InvalidReturnDay {
        day_borrowed: 10,
        day_returned: 5,
    };

    assert_eq!(err1.to_string(), "item title cannot be empty");
    assert_eq!(err2.to_string(), "item with id 42 already exists");
    assert_eq!(err3.to_string(), "invalid return day 5: borrowed on day 10");
}
