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

    // A complete loan: Ada borrows "Dune" and returns it on time.
    library.checkout(1, 100, 0)?;
    let fee = library.return_item(1, 10)?;
    println!("Ada returned \"Dune\" on time and owes {fee} cents.");

    // A late return: audiobooks may be kept 14 days; this one comes back on
    // day 34, so it is 20 days overdue at 25 cents a day.
    library.checkout(2, 100, 0)?;
    let fee = library.return_item(2, 34)?;
    println!("Ada returned \"Project Hail Mary\" late and owes {fee} cents.");

    // A handled error: checking out an item that was never stocked. `main`
    // could propagate this with `?`, but the assignment wants one error
    // printed via its `Display` message instead of crashing the demo.
    match library.checkout(999, 100, 0) {
        Ok(()) => println!("unexpected success"),
        Err(error) => println!("Handled error: {error}"),
    }

    Ok(())
}

// Part 7, Experiment A: `add_item` takes `item: Item` by value, so the call
// moves it into the library. Reading `item.title` afterwards uses a value
// that no longer belongs to this scope.
#[allow(dead_code)]
fn experiment_a(library: &mut Library) -> Result<(), LibraryError> {
    let item = Item::new(
        5,
        "Experiment".into(),
        "Author".into(),
        MediaKind::Book { pages: 1 },
    );
    library.add_item(item)?;
    // println!("{}", item.title); // E0382: borrow of moved value `item`
    Ok(())
}

// Part 7, Experiment B: `find_item` returns `Option<&Item>`, an immutable
// borrow of `library`. `checkout` needs `&mut library`. Holding `held` across
// the `checkout` call keeps the immutable borrow alive while the mutable one
// is requested.
#[allow(dead_code)]
fn experiment_b(library: &mut Library) -> Result<(), LibraryError> {
    let _held = library.find_item(1);
    library.checkout(1, 100, 0)?;
    // println!("{_held:?}"); // E0502: cannot borrow `*library` as mutable
    // because it is also borrowed as immutable (via `_held`, held live to here)
    Ok(())
}
