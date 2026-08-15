//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{LibraryError, Item, Library, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
    // TODO(Part 8): stock a library, register a member, run a loan and a late
    // return, then print one handled error using its `Display` message.
    let mut library = Library::new();

    library.add_item(Item::new(
        1,
        "Dune".into(),
        "Frank Herbert".into(),
        MediaKind::Book { pages: 320 },
    ))?;
    library.add_item(Item::new(
        2,
        "Project Hail Mary".into(),
        "Andy Weir".into(),
        MediaKind::Audiobook { minutes: 540 },
    ))?;
    library.add_item(Item::new(
        3,
        "The Rust Programming Language".into(),
        "Steve Klabnik".into(),
        MediaKind::Ebook { size_kb: 1_200 },
    ))?;

    library.register_member(Member::new(100, "Ada".into()))?;

    library.checkout(1, 100, 5)?;
    println!("checked out: {}", library.find_item(1).unwrap());

    let fee = library.return_item(1, 30)?;
    println!("returned late, fee owed: {fee} cents");
    println!("now: {}", library.find_item(1).unwrap());

    // A deliberate, handled error: item 1 was already returned above, so it's
    // no longer on loan. We print it rather than propagate it with `?`.
    match library.return_item(1, 31) {
        Ok(_) => unreachable!("item was already returned"),
        Err(error) => println!("handled error: {error}"),
    }

    Ok(())
}
