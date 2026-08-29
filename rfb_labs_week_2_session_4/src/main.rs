//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{Item, Library, LibraryError, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
    let mut library = Library::new();

    // 1. Stock the library with items
    let book = Item::new(
        1,
        "Dune".into(),
        "Frank Herbert".into(),
        MediaKind::Book { pages: 412 },
    );
    let audiobook = Item::new(
        2,
        "Project Hail Mary".into(),
        "Andy Weir".into(),
        MediaKind::Audiobook { minutes: 960 },
    );
    let ebook = Item::new(
        3,
        "The Rust Programming Language".into(),
        "Steve Klabnik".into(),
        MediaKind::Ebook { size_kb: 2400 },
    );

    // Part 7 Experiment A (commented out after recording error in README.md)
    // println!("Item title: {}", book.title);

    library.add_item(book)?;
    library.add_item(audiobook)?;
    library.add_item(ebook)?;

    // 2. Register a member
    let member = Member::new(100, "Ada Lovelace".into());
    library.register_member(member)?;

    println!("=== Community Lending Library Stocked ===");
    if let Some(item) = library.find_item(1) {
        println!("Found item: {item}");
    }

    // Part 7 Experiment B (commented out after recording error in README.md)
    // let held_item = library.find_item(1);
    // library.checkout(1, 100, 1)?;
    // println!("Held item: {:?}", held_item);

    // 3. Run a complete loan
    println!("\n--- Checking out item #1 (Dune) to Ada Lovelace (Member #100) on day 5 ---");
    library.checkout(1, 100, 5)?;
    if let Some(item) = library.find_item(1) {
        println!("Updated status: {item}");
    }

    // 4. Run a late return (Book allowed 21 days: day 5 to day 35 = 30 days held, 9 days late)
    println!("\n--- Returning item #1 on day 35 (Late Return) ---");
    let fee_cents = library.return_item(1, 35)?;
    println!(
        "Item returned successfully. Late fee charged: {fee_cents} cents (${:.2})",
        fee_cents as f64 / 100.0
    );

    if let Some(item) = library.find_item(1) {
        println!("Final status: {item}");
    }

    // 5. Print one handled error using its Display message
    println!("\n--- Attempting invalid checkout (Item #999 does not exist) ---");
    match library.checkout(999, 100, 36) {
        Ok(_) => println!("Unexpected checkout success"),
        Err(err) => println!("Handled error Display output: \"{err}\""),
    }

    Ok(())
}
