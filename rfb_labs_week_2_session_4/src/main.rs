//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{Item, Library, LibraryError, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
     // TODO(Part 8): stock a library, register a member, run a loan and a late
    // return, then print one handled error using its `Display` message.
    println!("nothing to see yet — start at Part 1");
    let mut library = Library::new();

    // Stock the library
    library.add_item(Item::new(
        1,
        "Eat That Frog!".into(),
        "Brain Tracy".into(),
        MediaKind::Book { pages: 328 },
    ))?;
    library.add_item(Item::new(
        2,
        "How to Talk to Anyone, Anytime, Anywhere".into(),
        "Larry King".into(),
        MediaKind::Audiobook { minutes: 600 },
    ))?;
    library.add_item(Item::new(
        3,
        "Think Like A Billionaire".into(),
        "Scott Anderson".into(),
        MediaKind::Ebook { size_kb: 850 },
    ))?;

    // Register a member
    library.register_member(Member::new(101, "Alice".into()))?;

    // Check out an item (book, kept 21 days)
    println!("Alice checks out 1984...");
    library.checkout(1, 101, 10)?;
    println!("✓ Item checked out successfully");

    // Return the item late (kept 30 days when max is 21 = 9 days late = 9 * 25 = 225 cents)
    println!("\nAlice returns 1984 late (after 30 days)...");
    let fee = library.return_item(1, 40)?;
    println!(
        "✓ Item returned. Late fee: {} cents (${}.{})",
        fee,
        fee / 100,
        fee % 100
    );

    // Try to register a duplicate member (this will error)
    println!("\nAttempting to register Alice again...");
    match library.register_member(Member::new(101, "Alice Again".into())) {
        Ok(()) => println!("✓ Registered"),
        Err(e) => println!("✗ Error: {}", e),
    }

    Ok(())
}
