//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{Item, Library, LibraryError, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
    // (Part 8): stock a library, register a member, run a loan and a late
    // return, then print one handled error using its `Display` message.

    let mut library = Library::new();
    //We create an instance of Item
    let item_1 = Item::new(
        6,
        "Rust for Bitcoin".into(),
        "Julius".into(),
        MediaKind::Book { pages: 60 },
    );

    let item_2 = Item::new(
        5,
        "Bitcoin".into(),
        "Jay".into(),
        MediaKind::Ebook { size_kb: 24 },
    );

    library.add_item(item_1)?;
    library.add_item(item_2)?;

    //creating a member
    let member_1 = Member::new(1, "Sharon".into());
    let member_2 = Member::new(2, "Prince".into());

    library.register_member(member_1)?;
    library.register_member(member_2)?;

    library.checkout(6, 1, 1)?;
    let return_instance = library.return_item(6, 25)?;

    println!("Fee owed after return: {return_instance}");

    //handling error via display
    match library.checkout(5, 1, 30) {
        Ok(()) => println!("Checked out item 5"),
        Err(err) => println!("Handled Error: {err}"),
    }

    match library.checkout(999, 1, 30) {
        Ok(()) => println!("Checked out item 999"),
        Err(err) => println!("Handled Err: {err}"),
    }

    // let item_3 =
    //  Item::new(7, "btcacademy".into(), "Jeremiah".into(), MediaKind::Book { pages: 255 });
    // library.add_item(item_3)?;
    // println!("{}", item_3.title); //  triggers the error

    // let found = library.find_item(6);
    // library.checkout(6, 1, 1)?;
    // println!("{}", found.unwrap().title); // triggers the error

    Ok(())
}
