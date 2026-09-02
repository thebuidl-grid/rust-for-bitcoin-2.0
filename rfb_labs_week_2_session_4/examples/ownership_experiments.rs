//! Part 7 — two deliberate borrow-checker failures.
//!
//! Each experiment was written out in full, run with
//! `cargo check --example ownership_experiments`, and the real compiler error
//! pasted into README.md. The offending lines are commented out below, with
//! the version that does compile kept alongside so the contrast is visible.

use rfb_labs_week_2_session_4::{Item, Library, LibraryError, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
    experiment_a()?;
    experiment_b()?;
    Ok(())
}

/// **A** — read `item.title` after `library.add_item(item)?`.
///
/// Fails with E0382: `add_item` takes the `Item` by value, so the local
/// binding is dead the moment the call returns.
fn experiment_a() -> Result<(), LibraryError> {
    let mut library = Library::new();
    let item = Item::new(
        1,
        "Dune".to_string(),
        "Frank Herbert".to_string(),
        MediaKind::Book { pages: 320 },
    );

    library.add_item(item)?;

    // error[E0382]: borrow of moved value: `item`
    // println!("stocked {}", item.title);

    // The library owns the title now, so ask the library for it.
    if let Some(stocked) = library.find_item(1) {
        println!("A: stocked {}", stocked.title);
    }

    Ok(())
}

/// **B** — hold the result of `library.find_item(1)`, call
/// `library.checkout(..)?`, then print what was held.
///
/// Fails with E0502: the held `&Item` keeps `library` immutably borrowed
/// across a call that needs `&mut library`.
fn experiment_b() -> Result<(), LibraryError> {
    let mut library = Library::new();
    library.add_item(Item::new(
        1,
        "Dune".to_string(),
        "Frank Herbert".to_string(),
        MediaKind::Book { pages: 320 },
    ))?;
    library.register_member(Member::new(100, "Ada".to_string()))?;

    // error[E0502]: cannot borrow `library` as mutable because it is also
    // borrowed as immutable
    // let held = library.find_item(1);
    // library.checkout(1, 100, 5)?;
    // println!("held {held:?}");

    // Copy out what is needed, or simply look the item up again afterwards —
    // the second borrow starts after the mutation has finished.
    library.checkout(1, 100, 5)?;
    if let Some(borrowed) = library.find_item(1) {
        println!("B: {borrowed}");
    }

    Ok(())
}
