//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{Item, Library, LibraryError, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
    // TODO(Part 8): stock a library, register a member, run a loan and a late
    // return, then print one handled error using its `Display` message.

    // Create a new library
    let mut library = Library::new();
    // Stock the library with various items
    library.add_item(Item::new(
        1,
        "Dune".to_string(),
        "Frank Herbert".to_string(),
        MediaKind::Book { pages: 320 },
    ))?;

    library.add_item(Item::new(
        2,
        "Project Hail Mary".to_string(),
        "Andy Weir".to_string(),
        MediaKind::Audiobook { minutes: 540 },
    ))?;

    library.add_item(Item::new(
        3,
        "The Rust Programming Language".to_string(),
        "Steve Klabnik".to_string(),
        MediaKind::Ebook { size_kb: 1_200 },
    ))?;

    library.add_item(Item::new(
        4,
        "Children of Dune".to_string(),
        "Frank Herbert".to_string(),
        MediaKind::Book { pages: 180 },
    ))?;

    // Register a member

    library.register_member(Member::new(100, "Ada Lovelace".to_string()))?;
    library.register_member(Member::new(101, "Grace Hopper".to_string()))?;

    // Perform a successful checkout

    library.checkout(1, 100, 10)?;

    if let Some(item) = library.find_item(1) {
        println!("  status: {}", item.status);
    }

    let fee = library.return_item(1, 40)?;
    println!(
        "  Late fee charged: {} cents (${:.2})",
        fee,
        fee as f64 / 100.0
    );

    if let Some(item) = library.find_item(1) {
        println!("  New status: {}", item.status);
    }
    println!();

    // Demonstrate an on-time return with ebook (no late fees ever)
    library.checkout(3, 101, 20)?;
    let ebook_fee = library.return_item(3, 35)?; // 15 days, well over 7 day limit
    println!(
        "  Fee: {} cents (ebooks never charge late fees!)",
        ebook_fee
    );

    // Demonstrate error handling by triggering an error
    match library.checkout(999, 100, 50) {
        Ok(_) => println!("unexpected success"),
        Err(e) => println!("error: {}", e),
    }
    println!();

    // Demonstrate another error: trying to checkout already borrowed item
    library.checkout(2, 100, 50)?;
    match library.checkout(2, 101, 51) {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("error: {}", e),
    }

    // Show borrow limit
    library.checkout(4, 100, 60)?;
    library.checkout(3, 100, 60)?;
    // Member 100 now has 3 items (2, 4, 3)

    // Try to borrow a 4th item (should fail)
    library.add_item(Item::new(
        5,
        "1984".to_string(),
        "George Orwell".to_string(),
        MediaKind::Book { pages: 328 },
    ))?;

    match library.checkout(5, 100, 61) {
        Ok(_) => println!("unexpected success"),
        Err(e) => println!("error: {}", e),
    }
    println!();

    Ok(())
}

//println!("nothing to see yet — start at Part 1");
