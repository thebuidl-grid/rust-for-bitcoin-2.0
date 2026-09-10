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

// ========== TESTES ADICIONAIS DE ADIÇÃO DE ITENS ==========

#[test]
fn add_item_with_empty_title_fails() {
    let mut library = Library::new();
    let item = Item::new(
        1,
        String::new(),
        "Author".into(),
        MediaKind::Book { pages: 100 },
    );

    assert_eq!(library.add_item(item), Err(LibraryError::EmptyTitle));
}

#[test]
fn add_duplicate_item_id_fails() {
    let mut library = Library::new();
    let item1 = Item::new(
        1,
        "Title 1".into(),
        "Author 1".into(),
        MediaKind::Book { pages: 100 },
    );
    let item2 = Item::new(
        1,
        "Title 2".into(),
        "Author 2".into(),
        MediaKind::Book { pages: 200 },
    );

    library.add_item(item1).unwrap();
    assert_eq!(
        library.add_item(item2),
        Err(LibraryError::DuplicateItemId { id: 1 })
    );
}

#[test]
fn add_item_successfully() {
    let mut library = Library::new();
    let item = Item::new(
        1,
        "Title".into(),
        "Author".into(),
        MediaKind::Book { pages: 100 },
    );

    assert!(library.add_item(item).is_ok());
    assert_eq!(library.find_item(1).unwrap().title, "Title");
}

// ========== TESTES ADICIONAIS DE REGISTRO DE MEMBROS ==========

#[test]
fn register_member_successfully() {
    let mut library = Library::new();
    let member = Member::new(1, "John".into());

    assert!(library.register_member(member).is_ok());
    assert_eq!(library.find_member(1).unwrap().name, "John");
}

#[test]
fn register_duplicate_member_id_fails() {
    let mut library = Library::new();
    let member1 = Member::new(1, "John".into());
    let member2 = Member::new(1, "Jane".into());

    library.register_member(member1).unwrap();
    assert_eq!(
        library.register_member(member2),
        Err(LibraryError::DuplicateMemberId { id: 1 })
    );
}

// ========== TESTES ADICIONAIS DE BUSCA ==========

#[test]
fn find_nonexistent_item_returns_none() {
    let library = library_with_items();
    assert_eq!(library.find_item(999), None);
}

#[test]
fn find_nonexistent_member_returns_none() {
    let library = library_with_items();
    assert_eq!(library.find_member(999), None);
}

#[test]
fn items_by_author_returns_multiple_items() {
    let library = library_with_items();
    let found = library.items_by_author("Frank Herbert");

    assert_eq!(found.len(), 2);
    assert_eq!(found[0].title, "Dune");
    assert_eq!(found[1].title, "Children of Dune");
}

#[test]
fn items_by_author_no_matches() {
    let library = library_with_items();
    let found = library.items_by_author("Unknown Author");

    assert_eq!(found.len(), 0);
}

// ========== TESTES ADICIONAIS DE ITENS DISPONÍVEIS ==========

#[test]
fn available_items_initially_all_available() {
    let library = library_with_items();
    let available = library.available_items();

    assert_eq!(available.len(), 4);
}

#[test]
fn available_items_excludes_loaned_items() {
    let mut library = library_with_items();
    library.checkout(1, 100, 0).unwrap();
    library.checkout(2, 100, 0).unwrap();

    let available = library.available_items();
    assert_eq!(available.len(), 2);
    assert!(!available.iter().any(|item| item.id == 1 || item.id == 2));
}

// ========== TESTES ADICIONAIS DE EMPRÉSTIMO MAIS LONGO ==========

#[test]
fn longest_loan_item_returns_book() {
    let library = library_with_items();
    let longest = library.longest_loan_item();

    // Book: 21 days, Audiobook: 14 days, Ebook: 7 days
    let item = longest.unwrap();
    assert_eq!(item.loan_days(), 21); // Qualquer livro tem 21 dias
    assert!(matches!(item.kind, MediaKind::Book { .. }));
}

#[test]
fn longest_loan_item_empty_library() {
    let library = Library::new();
    assert_eq!(library.longest_loan_item(), None);
}

// ========== TESTES ADICIONAIS DE CHECKOUT ==========

#[test]
fn checkout_nonexistent_item_fails() {
    let mut library = library_with_items();

    assert_eq!(
        library.checkout(999, 100, 0),
        Err(LibraryError::ItemNotFound { id: 999 })
    );
}

#[test]
fn checkout_nonexistent_member_fails() {
    let mut library = library_with_items();

    assert_eq!(
        library.checkout(1, 999, 0),
        Err(LibraryError::MemberNotFound { id: 999 })
    );
}

#[test]
fn checkout_lost_item_fails() {
    let mut library = Library::new();
    let mut item = Item::new(
        1,
        "Title".into(),
        "Author".into(),
        MediaKind::Book { pages: 100 },
    );
    item.status = LoanStatus::Lost;

    library.add_item(item).unwrap();
    library
        .register_member(Member::new(100, "Ada".into()))
        .unwrap();

    assert_eq!(
        library.checkout(1, 100, 0),
        Err(LibraryError::ItemIsLost { id: 1 })
    );
}

#[test]
fn checkout_already_loaned_item_fails() {
    let mut library = library_with_items();

    library.checkout(1, 100, 0).unwrap();

    assert_eq!(
        library.checkout(1, 100, 5),
        Err(LibraryError::ItemAlreadyOnLoan {
            id: 1,
            member_id: 100,
        })
    );
}

#[test]
fn multiple_members_can_checkout_different_items() {
    let mut library = library_with_items();
    let member2 = Member::new(101, "Bob".into());
    library.register_member(member2).unwrap();

    library.checkout(1, 100, 0).unwrap();
    library.checkout(2, 101, 0).unwrap();

    assert_eq!(library.find_member(100).unwrap().borrowed_item_ids, vec![1]);
    assert_eq!(library.find_member(101).unwrap().borrowed_item_ids, vec![2]);
}

// ========== TESTES ADICIONAIS DE RETORNO ==========

#[test]
fn return_item_on_time_charges_no_fee() {
    let mut library = library_with_items();

    library.checkout(1, 100, 10).unwrap();

    // Book pode ser mantido por 21 dias, retornado no dia 25 (15 dias após o empréstimo)
    assert_eq!(library.return_item(1, 25), Ok(0));
    assert_eq!(library.find_item(1).unwrap().status, LoanStatus::Available);
}

#[test]
fn returning_audiobook_late_charges_a_daily_fee() {
    let mut library = library_with_items();

    // Audiobook pode ser mantido 14 dias, taxa diária é 25 cents
    library.checkout(3, 100, 0).unwrap();

    assert_eq!(library.return_item(3, 20), Ok(6 * 25));
}

#[test]
fn returning_ebook_late_charges_no_fee() {
    let mut library = library_with_items();

    // Ebook pode ser mantido 7 dias, mas taxa diária é 0
    library.checkout(4, 100, 0).unwrap();

    assert_eq!(library.return_item(4, 30), Ok(0));
}

#[test]
fn return_nonexistent_item_fails() {
    let mut library = library_with_items();

    assert_eq!(
        library.return_item(999, 10),
        Err(LibraryError::ItemNotFound { id: 999 })
    );
}

#[test]
fn return_not_loaned_item_fails() {
    let mut library = library_with_items();

    assert_eq!(
        library.return_item(1, 10),
        Err(LibraryError::ItemNotOnLoan { id: 1 })
    );
}

#[test]
fn return_lost_item_fails() {
    let mut library = Library::new();
    let mut item = Item::new(
        1,
        "Title".into(),
        "Author".into(),
        MediaKind::Book { pages: 100 },
    );
    item.status = LoanStatus::Lost;

    library.add_item(item).unwrap();
    library
        .register_member(Member::new(100, "Ada".into()))
        .unwrap();

    assert_eq!(
        library.return_item(1, 10),
        Err(LibraryError::ItemIsLost { id: 1 })
    );
}

#[test]
fn return_before_checkout_day_fails() {
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

#[test]
fn return_item_removes_from_member_borrowed_list() {
    let mut library = library_with_items();

    library.checkout(1, 100, 0).unwrap();
    library.checkout(2, 100, 0).unwrap();

    library.return_item(1, 10).unwrap();

    assert_eq!(library.find_member(100).unwrap().borrowed_item_ids, vec![2]);
}

#[test]
fn return_item_with_same_checkout_and_return_day() {
    let mut library = library_with_items();

    library.checkout(1, 100, 10).unwrap();

    // Retornado no mesmo dia é permitido (0 dias de atraso)
    assert_eq!(library.return_item(1, 10), Ok(0));
}

// ========== TESTES ADICIONAIS DE FLUXO COMPLETO ==========

#[test]
fn complete_loan_cycle() {
    let mut library = library_with_items();
    let member2 = Member::new(101, "Bob".into());
    library.register_member(member2).unwrap();

    // Ada pega 3 itens
    library.checkout(1, 100, 0).unwrap();
    library.checkout(2, 100, 5).unwrap();
    library.checkout(3, 100, 10).unwrap();

    // Bob pega 1 item
    library.checkout(4, 101, 8).unwrap();

    // Ada devolve um item
    let fee = library.return_item(1, 30).unwrap();
    assert_eq!(fee, 9 * 25); // 9 dias atrasado

    // Ada agora pode pegar outro item
    let item5 = Item::new(
        5,
        "New Book".into(),
        "Author".into(),
        MediaKind::Book { pages: 300 },
    );
    library.add_item(item5).unwrap();
    assert!(library.checkout(5, 100, 32).is_ok());
}

#[test]
fn member_after_returning_item_can_checkout_again() {
    let mut library = library_with_items();

    library.checkout(1, 100, 0).unwrap();
    library.return_item(1, 10).unwrap();

    // Deve ser possível emprestar novamente
    assert!(library.checkout(1, 100, 15).is_ok());
}

// ========== TESTES ADICIONAIS DO MÉTODO FILTER_ITEMS ==========

#[test]
fn filter_items_with_custom_predicate() {
    let library = library_with_items();

    let books = library.filter_items(|item| matches!(item.kind, MediaKind::Book { .. }));

    assert_eq!(books.len(), 2);
    assert!(
        books
            .iter()
            .all(|item| matches!(item.kind, MediaKind::Book { .. }))
    );
}

#[test]
fn filter_items_no_matches() {
    let library = library_with_items();

    let filtered = library.filter_items(|item| item.id > 1000);

    assert_eq!(filtered.len(), 0);
}
