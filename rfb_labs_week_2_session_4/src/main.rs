//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{
    Library, LibraryError,
    catalogue::{Item, MediaKind},
};

fn main() -> Result<(), LibraryError> {
    let mut library = Library::new();

    library.add_item(Item::new(
        1,
        "Rust Book".to_string(),
        "Steve".to_string(),
        MediaKind::Book { pages: 500 },
    ))?;

    library.add_item(Item::new(
        2,
        "Rust Audio".to_string(),
        "Steve".to_string(),
        MediaKind::Audiobook { minutes: 300 },
    ))?;

    library.register_member(rfb_labs_week_2_session_4::member::Member::new(
        1,
        "Rose".to_string(),
    ))?;

    // Borrow on day 1.
    library.checkout(1, 1, 1)?;

    // Return on day 25: book allows 21 days, so 4 late days × 25 cents.
    let fee = library.return_item(1, 25)?;

    println!("Returned item 1 late. Fee: {} cents", fee);

    // Demonstrate a handled error.
    if let Err(error) = library.checkout(999, 1, 30) {
        println!("Handled error: {}", error);
    }

    Ok(())
}
