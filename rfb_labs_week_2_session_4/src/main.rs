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
    library.checkout(1, 100, 0)?;
    println!("checked out: {}", library.find_item(1).unwrap());
    let fee = library.return_item(1, 10)?;
    println!("returned on day 10, fee owed: {fee} cents");
    println!("item after return: {}", library.find_item(1).unwrap());

    // A late loan: an audiobook may be kept 14 days, held for 20.
    library.checkout(2, 100, 0)?;
    let late_fee = library.return_item(2, 20)?;
    println!("returned late on day 20, fee owed: {late_fee} cents");

    // One handled error, printed via its `Display` message.
    match library.checkout(1, 999, 0) {
        Ok(()) => println!("unexpectedly succeeded"),
        Err(error) => println!("handled error: {error}"),
    }

    Ok(())
}

#[allow(dead_code)]
fn experiment_a() -> Result<(), LibraryError> {
    let mut library = Library::new();
    let item = Item::new(
        99,
        "Foundation".into(),
        "Isaac Asimov".into(),
        MediaKind::Book { pages: 255 },
    );
    library.add_item(item)?;
    // Experiment A: `item` was moved into `add_item`, so it can no longer be
    // read here. See README.md for the real `cargo check` error.
    // println!("still have: {}", item.title);
    Ok(())
}

#[allow(dead_code)]
fn experiment_b() -> Result<(), LibraryError> {
    let mut library = Library::new();
    library.add_item(Item::new(
        1,
        "Dune".into(),
        "Frank Herbert".into(),
        MediaKind::Book { pages: 412 },
    ))?;
    library.register_member(Member::new(100, "Ada".into()))?;

    let held = library.find_item(1);
    // Experiment B: `held` keeps an immutable borrow of `library` alive, and
    // `checkout` needs `&mut library`. See README.md for the real error.
    // library.checkout(1, 100, 0)?;
    println!("still have: {:?}", held);
    Ok(())
}
