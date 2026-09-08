//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{Item, Library, LibraryError, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
    let mut library = Library::new();

    // Stock the library with a few items.
    library.add_item(Item::new(
        1,
        "Dune".into(),
        "Frank Herbert".into(),
        MediaKind::Book { pages: 320 },
    ))?;
    library.add_item(Item::new(
        2,
        "The Rust Programming Language".into(),
        "Steve Klabnik".into(),
        MediaKind::Ebook { size_kb: 1_200 },
    ))?;
    library.add_item(Item::new(
        3,
        "Project Hail Mary".into(),
        "Andy Weir".into(),
        MediaKind::Audiobook { minutes: 540 },
    ))?;

    // Register a member.
    library.register_member(Member::new(100, "Ada".into()))?;

    println!("=== Library stocked ===");
    for item in library.available_items() {
        println!("  {item}");
    }

    // First loan: borrow on day 0, return on day 20 (one day early — no fee).
    library.checkout(1, 100, 0)?;
    println!("\nAda checked out item 1 on day 0.");

    let fee = library.return_item(1, 20)?;
    println!("Ada returned item 1 on day 20. Fee owed: {fee} cents (on time).");

    // Late return: borrow on day 30, return on day 65 (35 days held; book limit 21 days).
    library.checkout(1, 100, 30)?;
    println!("\nAda checked out item 1 again on day 30.");

    let fee = library.return_item(1, 65)?;
    let days_late = 35u32.saturating_sub(21);
    println!("Ada returned item 1 on day 65. Fee owed: {fee} cents ({days_late} days late).");

    // Print a handled error using its Display message.
    library.checkout(1, 100, 70)?;
    let err = library.checkout(1, 100, 70).unwrap_err();
    println!("\nHandled error: {err}");

    Ok(())
}
