//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::Item;
use rfb_labs_week_2_session_4::Library;
use rfb_labs_week_2_session_4::LibraryError;
use rfb_labs_week_2_session_4::MediaKind;
use rfb_labs_week_2_session_4::Member;

fn main() -> Result<(), LibraryError> {
    // TODO(Part 8): stock a library, register a member, run a loan and a late
    // return, then print one handled error using its `Display` message.
    // println!("nothing to see yet — start at Part 1");

    // let item = Item::new(
    //     1,
    //     "Dune 2".into(),
    //     "Frank Herbert".into(),
    //     MediaKind::Book { pages: 390 },
    // );
    // let mut library = Library::new();
    // library.add_item(item)?;
    // println!("{}", item.title);

    // let mut library = Library::new();

    // let found = library.find_item(1);
    // library.checkout(1, 500, 5)?;
    // println!("{:?}", found);

    let mut library = Library::new();

    for (id, title, author, kind) in [
        (1, "Sapele", "Yinka Ayefele", MediaKind::Book { pages: 447 }),
        (
            2,
            "Ready Player One",
            "Hakeem Lyon",
            MediaKind::Book { pages: 250 },
        ),
        (
            3,
            "Mickey 17",
            "Sandra Bullock",
            MediaKind::Audiobook { minutes: 430 },
        ),
        (
            4,
            "The Rust Programming Language",
            "Steve Klabnik",
            MediaKind::Ebook { size_kb: 1_200 },
        ),
    ] {
        library
            .add_item(Item::new(id, title.into(), author.into(), kind))
            .unwrap();
    }

    library
        .register_member(Member::new(100, "Ada".into()))
        .unwrap();

    library.checkout(3, 100, 6)?;

    library.return_item(3, 35)?;

    if let Err(e) = library.add_item(Item::new(
        5,
        "".into(),
        "David Mark".into(),
        MediaKind::Ebook { size_kb: 1_580 },
    )) {
        println!("{e}")
    }

    Ok(())
}
