//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::LibraryError;
use rfb_labs_week_2_session_4::catalogue::{Item, MediaKind};
use rfb_labs_week_2_session_4::library::Library;
use rfb_labs_week_2_session_4::member::Member;

fn library_with_items() -> Library {
    let mut library = Library::new();

    for (id, title, author, kind) in [
        (1, "Dune", "Frank Herbert", MediaKind::Book { pages: 320 }),
        (
            2,
            "Children of Dune",
            "Frank Herbert",
            MediaKind::Book { pages: 180 },
        ),
        (
            3,
            "Project Hail Mary",
            "Andy Weir",
            MediaKind::Audiobook { minutes: 540 },
        ),
        (
            4,
            "The Rust Programming Language",
            "Steve Klabnik",
            MediaKind::Ebook { size_kb: 1_200 },
        ),
    ] {
        library
            .add_item(Item::new(id, title.into(), author.into(), kind))
            .unwrap();
    }

    library
        .register_member(Member::new(100, "Ada".into()))
        .unwrap();

    library
}

fn main() -> Result<(), LibraryError> {
    // TODO(Part 8): stock a library, register a member, run a loan and a late
    // return, then print one handled error using its `Display` message.
    // println!("nothing to see yet — start at Part 1");

    //     let mut library = Library::new();

    //     // Experiment A: Read item.title after moving `item` into library

    //     let item = Item::new(
    //         1,
    //         "Dune".into(),
    //         "Frank Herbert".into(),
    //         MediaKind::Book { pages: 412 },
    //     );
    //     library.add_item(item)?;

    //     println!("{}", item.title);

    //     // Experiment B: Mutate library while holding an immutable borrow

    //     let member = Member::new(101, "Alice".into());
    //     library.register_member(member)?;

    //     let item_ref = library.find_item(1);
    //     library.checkout(1, 101, 1)?;

    //    println!("{:?}", item_ref);

    // Stock library and register member via helper
    let mut library = library_with_items();
    println!("--- Library Stocked & Member Registered ---");

    // Complete loan (Ada borrows Dune on day 1)
    println!("\n--- Borrowing Item ---");
    library.checkout(1, 100, 1)?;
    if let Some(item) = library.find_item(1) {
        println!("{item}");
    }

    // Late return (Dune has 21 loan days; returned on day 30 -> 8 days late = 200 cents)
    println!("\n--- Returning Item (Late) ---");
    let fee_cents = library.return_item(1, 30)?;
    println!(
        "Returned item 1 on day 30. Late fee owed: {fee_cents} cents (${:.2})",
        fee_cents as f64 / 100.0
    );

    // Print one handled error using its Display message
    println!("\n--- Handling an Error ---");
    match library.return_item(1, 5) {
        Ok(_) => println!("Unexpected success!"),
        Err(err) => println!("Handled expected error: {err}"),
    }

    Ok(())
}
