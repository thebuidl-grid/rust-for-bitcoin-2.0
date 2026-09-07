//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{Item, Library, LibraryError, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
    let mut library = Library::new();

    let item = Item::new(
        1,
        "The Great Gatsby".to_string(),
        "F. Scott Fitzgerald".to_string(),
        MediaKind::Book { pages: 180 },
    );

    let member = Member::new(1, "Andres".to_string());

    // Stock the library and register a member.
    library.add_item(item)?;
    library.register_member(member)?;

    // Borrow the book on day 10.
    library.checkout(1, 1, 10)?;

    // Return it late on day 35.
    let late_fee = library.return_item(1, 35)?;

    println!("Late fee: {late_fee} cents");

    // Try to return the same item again.
    // This is an expected error, so we handle it instead of panicking.
    match library.return_item(1, 40) {
        Ok(fee) => {
            println!("Returned successfully. Late fee: {fee} cents");
        }
        Err(error) => {
            println!("Handled error: {error}");
        }
    }

    Ok(())
}
