//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{Item, Library, LibraryError, LoanTerms, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
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
        library.add_item(Item::new(id, title.into(), author.into(), kind))?;
    }

    library.register_member(Member::new(100, "Ada".into()))?;

    println!("== catalogue ==");
    for item in library.available_items() {
        println!("  {item}  (may be kept {} days)", item.loan_days());
    }

    if let Some(item) = library.longest_loan_item() {
        println!(
            "\nlongest loan: {} at {} days",
            item.title,
            item.loan_days()
        );
    }

    // A complete loan, borrowed on day 10 and kept well past its 21 days.
    println!("\n== loan ==");
    library.checkout(1, 100, 10)?;
    if let Some(item) = library.find_item(1) {
        println!("  {item}");
    }
    if let Some(member) = library.find_member(100) {
        println!("  {} now holds {:?}", member.name, member.borrowed_item_ids);
    }

    println!("\n== late return on day 40 ==");
    let fee_cents = library.return_item(1, 40)?;
    println!(
        "  30 days held against a 21 day loan — {} days overdue, {} owed",
        30 - 21,
        format_cents(fee_cents)
    );
    if let Some(item) = library.find_item(1) {
        println!("  {item}");
    }

    // Part 8 also wants one handled error, printed through `Display` rather
    // than unwrapped. Returning an item nobody borrowed is a natural one.
    println!("\n== a handled error ==");
    match library.return_item(2, 41) {
        Ok(fee) => println!("  unexpectedly owed {}", format_cents(fee)),
        Err(error) => println!("  could not return item 2: {error}"),
    }

    Ok(())
}

/// Whole cents into something a member would recognise on a receipt.
fn format_cents(cents: u32) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}
