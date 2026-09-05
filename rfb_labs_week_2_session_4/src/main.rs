//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{
    Item, Library, LibraryError, LoanTerms, MAX_ITEMS_PER_MEMBER, MediaKind, Member,
};

const ADA: u32 = 100;

fn main() -> Result<(), LibraryError> {
    let mut library = stocked_library()?;
    library.register_member(Member::new(ADA, "Ada".into()))?;

    println!("== Catalogue ==");
    for item in library.available_items() {
        println!("  {item}");
    }

    if let Some(longest) = library.longest_loan_item() {
        println!(
            "\nKept longest: {} — {} days",
            longest.title,
            longest.loan_days()
        );
    }

    // A complete loan, returned inside the audiobook's 14-day term.
    println!("\n== An on-time loan ==");
    library.checkout(3, ADA, 0)?;
    let on_loan = library
        .find_item(3)
        .ok_or(LibraryError::ItemNotFound { id: 3 })?;
    println!("  {on_loan}");
    let owed = library.return_item(3, 10)?;
    println!("  returned on day 10, owing {}", cents(owed));

    // The same loan run past its due date.
    println!("\n== A late return ==");
    library.checkout(1, ADA, 10)?;
    let owed = library.return_item(1, 40)?;
    println!("  borrowed day 10, returned day 40, owing {}", cents(owed));

    // An ebook is never overdue, however long it is held.
    println!("\n== An ebook, very late ==");
    library.checkout(4, ADA, 0)?;
    let owed = library.return_item(4, 365)?;
    println!("  held for a year, owing {}", cents(owed));

    // One handled error, printed through its `Display` impl.
    println!("\n== A handled error ==");
    library.checkout(2, ADA, 50)?;
    match library.checkout(2, ADA, 50) {
        Ok(()) => println!("  unexpectedly lent the same book twice"),
        Err(error) => println!("  {error}"),
    }

    let ada = library
        .find_member(ADA)
        .ok_or(LibraryError::MemberNotFound { id: ADA })?;
    println!(
        "\n{} is holding {} of {MAX_ITEMS_PER_MEMBER} items.",
        ada.name,
        ada.borrowed_item_ids.len()
    );

    ownership_experiments()?;

    Ok(())
}

fn stocked_library() -> Result<Library, LibraryError> {
    let mut library = Library::new();

    library.add_item(Item::new(
        1,
        "Dune".into(),
        "Frank Herbert".into(),
        MediaKind::Book { pages: 320 },
    ))?;
    library.add_item(Item::new(
        2,
        "Children of Dune".into(),
        "Frank Herbert".into(),
        MediaKind::Book { pages: 180 },
    ))?;
    library.add_item(Item::new(
        3,
        "Project Hail Mary".into(),
        "Andy Weir".into(),
        MediaKind::Audiobook { minutes: 540 },
    ))?;
    library.add_item(Item::new(
        4,
        "The Rust Programming Language".into(),
        "Steve Klabnik".into(),
        MediaKind::Ebook { size_kb: 1_200 },
    ))?;

    Ok(library)
}

/// Whole cents rendered as money, e.g. `225` becomes `$2.25`.
fn cents(amount: u32) -> String {
    format!("${}.{:02}", amount / 100, amount % 100)
}

/// Part 7. Each experiment's offending line is commented out; the recorded
/// compiler errors and their explanations are in README.md.
fn ownership_experiments() -> Result<(), LibraryError> {
    let mut library = Library::new();
    library.register_member(Member::new(ADA, "Ada".into()))?;

    // Experiment A — E0382, borrow of moved value. `add_item` takes the item
    // by value, so the local `item` is dead after the call.
    let item = Item::new(
        1,
        "Dune".into(),
        "Frank Herbert".into(),
        MediaKind::Book { pages: 320 },
    );
    library.add_item(item)?;
    // println!("still mine? {}", item.title);

    // Experiment B — E0502, mutable borrow while a shared borrow is live.
    // `held` borrows from `library`, and `checkout` needs `&mut library`.
    let _held = library.find_item(1);
    library.checkout(1, ADA, 0)?;
    // println!("held: {_held:?}");

    Ok(())
}
