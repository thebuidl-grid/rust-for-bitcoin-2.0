//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{Item, Library, LibraryError, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
    // TODO(Part 8): stock a library, register a member, run a loan and a late
    // return, then print one handled error using its `Display` message.

    let mut library = Library::new();

    library.add_item(Item::new(
        1,
        "Things Fall Apart".to_string(),
        "Chinua Achebe".to_string(),
        MediaKind::Book { pages: 1000 },
    ))?;
    library.add_item(Item::new(
        2,
        "The Richest Man in Babylon".to_string(),
        "George Samuel Clason".to_string(),
        MediaKind::Ebook { size_kb: 1280 },
    ))?;

    library.register_member(Member::new(7, "Xoulomon".to_string()))?;
    library.register_member(Member::new(8, "HIM".to_string()))?;

    let item_1 = library.find_item(1).unwrap();
    println!("Item 1: {:#?}", item_1);

    //Borrowed on day 5, returned day 15- within the 21-days , so no fee.
    library.checkout(1, 7, 5)?;
    let on_time_fee = library.return_item(1, 15)?;
    println!("On-time return fee: {on_time_fee} cents");

    // Borrowed on day 0, returned day 30 — 9 days past the 21-day term.
    library.checkout(1, 7, 0)?;
    let late_fee = library.return_item(1, 30)?;
    println!("Late return fee: {late_fee} cents");

    // Deliberate error: item 2 is checked out twice without a return in between.
    library.checkout(2, 7, 0)?;
    match library.checkout(2, 8, 0) {
        Ok(()) => println!("unexpected success"),
        Err(err) => println!("Handled error: {err}"),
    }

    Ok(())
}
