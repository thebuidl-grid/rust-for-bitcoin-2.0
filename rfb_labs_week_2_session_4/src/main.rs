//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{Item, Library, LibraryError, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
    let mut library = Library::new();

    library.add_item(Item::new(
        1,
        "The Rust Programming Language".into(),
        "Steve Klabnik".into(),
        MediaKind::Book { pages: 560 },
    ))?;
    library.add_item(Item::new(
        2,
        "Rust for Rustaceans".into(),
        "Jon Gjengset".into(),
        MediaKind::Ebook { size_kb: 2_400 },
    ))?;
    library.register_member(Member::new(100, "Ada".into()))?;

    library.checkout(1, 100, 10)?;
    println!("{}", library.find_item(1).expect("item was just added"));

    let fee = library.return_item(1, 40)?;
    println!("late return fee: {fee} cents");

    if let Err(error) = library.checkout(99, 100, 40) {
        println!("handled error: {error}");
    }

    Ok(())
}
