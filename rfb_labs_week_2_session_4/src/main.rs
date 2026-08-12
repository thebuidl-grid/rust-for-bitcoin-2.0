//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{Item, Library, LibraryError, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
    let mut library = Library::new();

    library.add_item(Item::new(
        1,
        "superstory".into(),
        "jide kosoko".into(),
        MediaKind::Book { pages: 412 },
    ))?;
    library.add_item(Item::new(
        2,
        "things fall apart".into(),
        "chinua achebe".into(),
        MediaKind::Audiobook { minutes: 970 },
    ))?;

    library.register_member(Member::new(100, "Ada".into()))?;

    println!("Stocked the library:");
    for item in library.available_items() {
        println!("  {item}");
    }

    // A complete loan: Ada borrows "superstory" on day 10.
    library.checkout(1, 100, 10)?;
    println!(
        "\nAda checked out item 1: {}",
        library.find_item(1).unwrap()
    );

    // A book may be kept 21 days; returning on day 40 is 9 days late.
    let fee = library.return_item(1, 40)?;
    println!("Ada returned item 1 on day 40 and owes {fee} cents in late fees.");

    // Trigger a handled error — member 999 was never registered — and print
    // it via its `Display` message instead of unwrapping or panicking.
    match library.checkout(2, 999, 0) {
        Ok(()) => println!("unexpectedly succeeded"),
        Err(error) => println!("\nhandled error: {error}"),
    }

    ownership_experiments();

    Ok(())
}

/// Part 7. Each line below was uncommented once, checked with `cargo check`,
/// and the real compiler error it produced was pasted into README.md. Both
/// stay commented out here since neither one is meant to compile.
fn ownership_experiments() {
    // Experiment A: `add_item` takes `Item` by value, so it moves `item`
    // in. Reading `item.title` afterwards is a use of a moved value —
    // see README.md for the pasted E0382 error.
    let mut library = Library::new();
    let item = Item::new(
        1,
        "superstory".into(),
        "jide kosoko".into(),
        MediaKind::Book { pages: 412 },
    );
    library.add_item(item).unwrap();
    // println!("{}", item.title);

    // Experiment B: `find_item` returns `&Item`, an immutable borrow of
    // `library`. `checkout` needs `&mut self`, and the immutable borrow is
    // still alive at the `println!` below — see README.md for the pasted
    // E0502 error.
    library
        .register_member(Member::new(100, "Ada".into()))
        .unwrap();
    let _found = library.find_item(1);
    library.checkout(1, 100, 0).unwrap();
    // println!("{:?}", found);
}
