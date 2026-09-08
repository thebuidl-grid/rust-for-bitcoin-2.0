//! Small executable for Part 8 of the assignment.

use rfb_labs_week_2_session_4::{Item, Library, LibraryError, LoanStatus, MediaKind, Member};

fn main() -> Result<(), LibraryError> {
    // TODO(Part 8): stock a library, register a member, run a loan and a late
    // return, then print one handled error using its `Display` message.

    let mut library = Library::new();
    let items = vec![
        Item {
            id: 0,
            title: "AudioBook".to_string(),
            author: "AudioBook Author".to_string(),
            kind: MediaKind::Audiobook { minutes: 23 },
            status: LoanStatus::Available,
        },
        Item {
            id: 1,
            title: "Book".to_string(),
            author: "Book Author".to_string(),
            kind: MediaKind::Book { pages: 32 },
            status: LoanStatus::Available,
        },
        Item {
            id: 2,
            title: "Ebook".to_string(),
            author: "Ebook author".to_string(),
            kind: MediaKind::Ebook { size_kb: 400 },
            status: LoanStatus::Available,
        },
    ];

    for item in items {
        library.add_item(item)?;
    }

    // The two experiments
    // let first_item_title = items[0].title.clone();
    // library.find_item(1);
    // library.checkout(1, 0, 30 )?;

    library.register_member(Member::new(0, "Test Member".to_string()))?;
    library.checkout(0, 0, 10)?;

    let fee = library.return_item(0, 30)?;
    println!("late fee: {} cents", fee);

    match library.checkout(9999, 0, 40) {
        Ok(_) => (),
        Err(err) => eprintln!("handled: {}", err),
    }

    Ok(())
}
