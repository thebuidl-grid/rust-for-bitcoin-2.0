//! Small executable for Part 8 of the assignment.
use rfb_labs_week_2_session_4::{Item, Library, LibraryError, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
    let mut library = Library::new();
    println!("Library created");
    let item = Item::new(
        1,
        "Rust for Rustaceans".to_string(),
        "Jon Gjengset".to_string(),
        MediaKind::Book { pages: 280 },
    );
    println!("Item created: {}", item);
    library.add_item(item)?;
    println!("Item added to Library");
    let member = Member::new(1, "Guilherme".to_string());
    println!("Member created: {:?}", member);

    library.register_member(member)?;
    println!("Member registered");

    println!("Adds a member already present:");
    match library.register_member(Member::new(1, "Guilherme".to_string())) {
        Ok(_) => panic!("member cannot be added 2 times"),
        Err(err) => println!("- Error: {err}"),
    }

    println!("Checks out an Item");
    library.checkout(1, 1, 5)?;

    println!("Return a Item late:");

    let fee = library.return_item(1, 50)?;

    println!("- Late fee: {}", fee);

    Ok(())
}
