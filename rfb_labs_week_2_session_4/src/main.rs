//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{Item, Library, LibraryError, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
    let mut library = Library::new();

    library.add_item(Item::new(
        1,
        "Dune".into(),
        "Frank Herbert".into(),
        MediaKind::Book { pages: 412 },
    ))?;
    library.add_item(Item::new(
        2,
        "Project Hail Mary".into(),
        "Andy Weir".into(),
        MediaKind::Audiobook { minutes: 970 },
    ))?;
    library.add_item(Item::new(
        3,
        "The Rust Programming Language".into(),
        "Steve Klabnik".into(),
        MediaKind::Ebook { size_kb: 1_200 },
    ))?;

    library.register_member(Member::new(100, "Ada".into()))?;

    // A complete, on-time loan.
    library.checkout(2, 100, 0)?;
    let fee = library.return_item(2, 5)?;
    println!("Ada returned item 2 on time, owing {fee} cents.");

    // A late loan: a book may be kept 21 days, so returning on day 30 is 9
    // days overdue at 25 cents a day.
    library.checkout(1, 100, 0)?;
    let fee = library.return_item(1, 30)?;
    println!("Ada returned item 1 late, owing {fee} cents.");

    for item in &[
        library.find_item(1).unwrap(),
        library.find_item(2).unwrap(),
        library.find_item(3).unwrap(),
    ] {
        println!("{item}");
    }

    // Trigger and print one handled error using its `Display` message.
    match library.checkout(999, 100, 0) {
        Ok(()) => unreachable!("item 999 does not exist"),
        Err(error) => println!("handled error: {error}"),
    }

    Ok(())
}
