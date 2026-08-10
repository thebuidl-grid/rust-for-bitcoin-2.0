//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{Item, Library, LibraryError, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
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

    // A complete loan: check the book out on day 0 and return it on time.
    library.checkout(1, 100, 0)?;
    let on_time_fee = library.return_item(1, 21)?;
    println!(
        "{} was returned on time; fee owed: {on_time_fee} cents",
        library.find_item(1).unwrap().title
    );

    // A late return: the audiobook may be kept 14 days, held for 20.
    library.checkout(2, 100, 0)?;
    let late_fee = library.return_item(2, 20)?;
    println!(
        "{} was returned 6 days late; fee owed: {late_fee} cents",
        library.find_item(2).unwrap().title
    );

    // Print one handled error using its `Display` message rather than
    // letting the program crash.
    match library.checkout(999, 100, 0) {
        Ok(()) => unreachable!("item 999 was never added"),
        Err(error) => println!("expected error checking out item 999: {error}"),
    }

    for id in [1, 2, 3] {
        println!("{}", library.find_item(id).unwrap());
    }

    // Experiment A (Part 7): `add_item` takes `item` by value, so the title
    // read below does not compile — `item` was moved into the library.
    // See README.md for the real `cargo check` error.
    let item = Item::new(
        5,
        "Foundation".into(),
        "Isaac Asimov".into(),
        MediaKind::Book { pages: 255 },
    );
    library.add_item(item)?;
    // println!("{}", item.title);

    // Experiment B (Part 7): holding an immutable borrow from `find_item`
    // across a mutable call to `checkout` does not compile.
    // See README.md for the real `cargo check` error.
    let _found = library.find_item(1);
    library.checkout(2, 100, 0)?;
    // println!("{}", _found.unwrap());

    Ok(())
}
