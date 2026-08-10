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
        "Rust in Action".into(),
        "Tim McNamara".into(),
        MediaKind::Ebook { size_kb: 1500 },
    ))?;

    library.register_member(Member::new(100, "Ada".into()))?;

    println!("Available items:");
    for item in library.available_items() {
        println!("  {item}");
    }

    library.checkout(1, 100, 10)?;
    println!("Checked out: {}", library.find_item(1).unwrap());

    let fee = library.return_item(1, 40)?;
    println!("Returned Dune on day 40; late fee: {fee} cents");

    match library.return_item(1, 40) {
        Ok(fee) => println!("Unexpected second return fee: {fee} cents"),
        Err(error) => println!("Handled error: {error}"),
    }

    Ok(())
}
