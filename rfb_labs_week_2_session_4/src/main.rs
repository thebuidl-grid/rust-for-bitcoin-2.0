//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{Item, Library, LibraryError, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
    // TODO(Part 8): stock a library, register a member, run a loan and a late
    // return, then print one handled error using its `Display` message.

    // let mut library = Library::new();
    // // let item = Item::new(1, "Dune".into(), "Frank Herbert".into(), MediaKind::Book { pages: 320 });
    // // library.add_item(item);
    // // println!("{}", item.title);

    // let found = library.find_item(1);
    // library.checkout(1, 100, 5)?;
    // println!("{:?}", found);

     let mut library = Library::new();

    // Stock the library
    library.add_item(Item::new(
        1,
        "Dune".into(),
        "Frank Herbert".into(),
        MediaKind::Book { pages: 320 },
    ))?;

    // Register a member
    library.register_member(Member::new(100, "Ada".into()))?;

    // Checkout on day 0
    library.checkout(1, 100, 0)?;
    println!("Checked out: {:?}", library.find_item(1).unwrap().status);

    // Return late — book is 21 days, returning on day 30 = 9 days overdue = 225 cents
    let fee = library.return_item(1, 30)?;
    println!("Late fee: {} cents", fee);
    println!("Status after return: {:?}", library.find_item(1).unwrap().status);

    // Print a handled error
    let error = LibraryError::ItemNotFound { id: 99 };
    println!("Handled error: {}", error);


    Ok(())
}
