//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{Item, Library, LibraryError, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
    let mut library = Library::new();

    // Stock library
    library.add_item(Item::new(
        1,
        "Mastering Bitcoin".into(),
        "Andreas Antonopoulos".into(),
        MediaKind::Book { pages: 398 },
    ))?;
    library.add_item(Item::new(
        2,
        "Programming Bitcoin".into(),
        "Jimmy Song".into(),
        MediaKind::Ebook { size_kb: 2500 },
    ))?;

    // Register member
    library.register_member(Member::new(101, "Elsuraj".into()))?;

    // Run loan checkout on day 10
    println!("--- Checking out Item 1 on day 10 ---");
    library.checkout(1, 101, 10)?;
    if let Some(item) = library.find_item(1) {
        println!("{item}");
    }

    // Run late return on day 40 (held 30 days; 9 days overdue @ 25c/day = 225 cents)
    println!("\n--- Returning Item 1 on day 40 ---");
    let fee = library.return_item(1, 40)?;
    println!("Returned item 1. Late fee owed: {fee} cents");

    // Print handled error using its Display message
    println!("\n--- Demonstrating Handled Error ---");
    match library.checkout(1, 999, 45) {
        Ok(_) => println!("Unexpected success"),
        Err(err) => println!("Handled error: {err}"),
    }

    Ok(())
}
