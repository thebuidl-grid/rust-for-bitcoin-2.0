//! Small executable for Part 8 of the assignment.
use rfb_labs_week_2_session_4::catalogue::{Item, MediaKind};
use rfb_labs_week_2_session_4::library::Library;
use rfb_labs_week_2_session_4::member::Member;
use rfb_labs_week_2_session_4::LibraryError;

fn main() -> Result<(), LibraryError> {
    let mut library = Library::new();

    // --- Stock the library with at least 5 items ---
    library.add_item(Item::new(
        1,
        "Dune".to_string(),
        "Frank Herbert".to_string(),
        MediaKind::Book { pages: 688 },
    ))?;
    library.add_item(Item::new(
        2,
        "Children of Dune".to_string(),
        "Frank Herbert".to_string(),
        MediaKind::Book { pages: 480 },
    ))?;
    library.add_item(Item::new(
        3,
        "The Hobbit".to_string(),
        "koko. milah".to_string(),
        MediaKind::Audiobook { minutes: 660 },
    ))?;
    library.add_item(Item::new(
        4,
        "Project Hail Mary".to_string(),
        "Abey shittu".to_string(),
        MediaKind::Ebook { size_kb: 2048 },
    ))?;
    library.add_item(Item::new(
        5,
        "Foundation".to_string(),
        "Isaac Asimov".to_string(),
        MediaKind::Book { pages: 255 },
    ))?;

    // --- Register a member ---
    library.register_member(Member::new(100, "Amaka".to_string()))?;

    println!("--- Catalogue on day 0 ---");
    for item in library.available_items() {
        println!("{item}");
    }

    // --- A complete, on-time loan ---
    let borrow_day = 10;
    library.checkout(1, 100, borrow_day)?;
    println!("\nChecked out item 1 to member 100 on day {borrow_day}.");

    let return_day_on_time = borrow_day + 5;
    let fee = library.return_item(1, return_day_on_time)?;
    println!(
        "Returned item 1 on day {return_day_on_time} (on time) — fee owed: {fee} cents."
    );

    // --- A late return ---
    let borrow_day_2 = 20;
    library.checkout(2, 100, borrow_day_2)?;
    println!("\nChecked out item 2 to member 100 on day {borrow_day_2}.");

    let return_day_late = borrow_day_2 + 25;
    let late_fee = library.return_item(2, return_day_late)?;
    println!(
        "Returned item 2 on day {return_day_late} (late) — fee owed: {late_fee} cents."
    );

    // --- Author search ---
    println!("\n--- Items by Frank Herbert ---");
    for item in library.items_by_author("Frank Herbert") {
        println!("{item}");
    }

    // --- Longest-loan item ---
    if let Some(item) = library.longest_loan_item() {
        println!("\nItem with the longest loan window: {item}");
    }

 // --- Print one handled error using its Display message ---
    match library.checkout(999, 100, 30) {
        Ok(()) => println!("\nUnexpectedly succeeded checking out item 999."),
        Err(error) => println!("\nHandled error as expected: {error}"),
    }

    Ok(())
}

