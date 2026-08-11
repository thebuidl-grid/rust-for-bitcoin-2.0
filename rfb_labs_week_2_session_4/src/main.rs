//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{Item, Library, LibraryError, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
    println!("=== Community Lending Library Demo ===");

    let mut library = Library::new();

    // Stock the library with items
    let book = Item::new(
        1,
        "Dune".into(),
        "Frank Herbert".into(),
        MediaKind::Book { pages: 320 },
    );
    let audiobook = Item::new(
        2,
        "Project Hail Mary".into(),
        "Andy Weir".into(),
        MediaKind::Audiobook { minutes: 540 },
    );
    let ebook = Item::new(
        3,
        "The Rust Programming Language".into(),
        "Steve Klabnik".into(),
        MediaKind::Ebook { size_kb: 1200 },
    );

    library.add_item(book)?;

    library.add_item(audiobook)?;
    library.add_item(ebook)?;

    println!("\nLibrary catalog stocked successfully:");
    for item in library.available_items() {
        println!(" - {item}");
    }

    // Register members
    let alice = Member::new(100, "Alice".into());
    let bob = Member::new(101, "Bob".into());
    library.register_member(alice)?;
    library.register_member(bob)?;
    println!("\nMembers registered: Alice (ID: 100), Bob (ID: 101)");

    // Longest loan item lookup
    if let Some(longest) = library.longest_loan_item() {
        println!("\nLongest allowable loan item: \"{}\"", longest.title);
    }

    // Run a checkout
    println!("\nChecking out item 1 (Dune) to member 100 on day 5...");
    // Experiment B: holding an immutable reference while calling checkout causes a borrow conflict.
    library.checkout(1, 100, 5)?;
    println!(
        "Checkout successful! Item status: {}",
        library.find_item(1).unwrap()
    );

    // Run a late return (book allowed for 21 days; returned on day 35 -> 30 days held, 9 days late * 25c = 225 cents = $2.25)
    println!("\nReturning item 1 on day 35 (overdue by 9 days)...");
    let fee = library.return_item(1, 35)?;
    println!(
        "Return successful! Late fee owed: {fee} cents (${:.2})",
        fee as f64 / 100.0
    );
    println!(
        "Item status after return: {}",
        library.find_item(1).unwrap()
    );

    // Demonstrate a handled validation error
    println!("\nAttempting invalid operation: returning an item not on loan (item 2)...");
    match library.return_item(2, 40) {
        Ok(_) => println!("Unexpected success!"),
        Err(err) => println!("Handled expected error using Display: \"{err}\""),
    }

    // Demonstrate another handled error: duplicate member registration
    println!("\nAttempting invalid operation: registering duplicate member ID 100...");
    match library.register_member(Member::new(100, "Alice Clone".into())) {
        Ok(_) => println!("Unexpected success!"),
        Err(err) => println!("Handled expected error using Display: \"{err}\""),
    }

    println!("\n=== Demo completed successfully ===");
    Ok(())
}
