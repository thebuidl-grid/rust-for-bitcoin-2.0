//! Part 7: Ownership Experiments
//! These demonstrate how Rust's ownership and borrow checker work.
//! Both experiments cause compiler errors, which are documented in README.md

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use crate::{Item, Library, MediaKind, Member};

    // Experiment A: Read item.title after library.add_item(item)?
    // COMMENTED OUT - Demonstrates move semantics
    // Uncomment to see: error[E0382]: borrow of moved value: `item`
    /*
    #[test]
    fn experiment_a_use_after_move() {
        let mut library = Library::new();
        let item = Item::new(1, "Test".into(), "Author".into(), MediaKind::Book { pages: 100 });
        library.add_item(item).ok();
        println!("{}", item.title);  // ERROR: item was moved into library
    }
    */

    // Experiment B: Hold the result of library.find_item(1),
    // call library.checkout(..)?, then print what you held
    // COMMENTED OUT - Demonstrates the borrow checker
    // Uncomment to see: error[E0502]: cannot borrow `library` as mutable
    /*
    #[test]
    fn experiment_b_cannot_hold_reference_during_mutation() {
        let mut library = Library::new();
        let item = Item::new(1, "Test".into(), "Author".into(), MediaKind::Book { pages: 100 });
        library.add_item(item).ok();
        library.register_member(Member::new(100, "Alice".into())).ok();

        let held = library.find_item(1);  // immutable borrow
        library.checkout(1, 100, 0).ok(); // ERROR: mutable borrow while immutable borrow active
        println!("{:?}", held);
    }
    */
}
