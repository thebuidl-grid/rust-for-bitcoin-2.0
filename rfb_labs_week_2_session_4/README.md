# Rust for Bitcoin 2.0 — Week 2, Session 4

Build a small lending library while practising structs, enums, traits,
ownership, borrowing, collections, and `Result`-based error handling. No
Bitcoin and no external crates — just Rust.

The crate is intentionally incomplete. Search for `TODO` and implement each
part; do not change the public type names or function signatures.

## Recommended workflow

1. Read [ASSIGNMENT.md](ASSIGNMENT.md).
2. Complete Part 2 in `error.rs`, then Part 3 in `library.rs`.
3. Remove `#[ignore]` from the relevant test and run it.
4. Complete the traits in Part 4 and the two operations in Parts 5–6.
5. Run the ownership experiments and record the errors.
6. Build the demo in `main.rs`.
7. Add the remaining required tests yourself.

```bash
cargo test
cargo test -- --ignored
cargo run
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

`cargo test` checks the starter project. Ignored tests intentionally exercise
unfinished code; enable them progressively rather than leaving them ignored in
the submission.

## Written answers

Answer in your own words. Add both ownership compiler errors from Part 7 as
fenced text blocks, then explain what caused each.

1. Why is `LoanStatus` an enum rather than a `bool` plus two `Option` fields?
- `LoanStatus` is an enum rather than a `bool` plus two `Option` fields because the struct alternative allows impossible states. For example: `is_on_loan: false` with `borrowed_by: Some(5)`, which is contradictory. The enum makes invalid states unrepresentable. Each state is self-contained and mutually exclusive.

2. What does `match` force you to do when a fourth `MediaKind` is added later?
- `match` statements allow you to handle every possible outcome of a particular type in this case, `MediaKind` enum, match statment handles every variant explicity also when a fourth variant of `MediaKind` is added, the `match` statement also handles it explicitly. 

3. `Item::new` takes `String` rather than `&str`. Who owns the title afterwards?
- `Item` owns the title since it's type is `String` which is an owned type not `&str` which is a reference

4. Why does `add_item` take `self` by `&mut` but `item` by value?
- The `add_item` function adds item to the library collection, in the process of adding item to the library collection, we are mutating `self` which is the `Library` struct. The `add_item` is taking ownership of the item passed into it as a parameter.

5. When `add_item` returns `Err`, what happened to the `Item` the caller passed in? Was that a good design choice, and what is the alternative?
The item was moved into add_item and then dropped — the caller loses it permanently even though the operation failed. This is not ideal. The alternative is to return the 
item back to the caller in the Err variant, e.g. Err((LibraryError::DuplicateItemId { id }, item)), so the caller can reuse it.

6. Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?
Because the library owns the items. Returning Option<Item> would move the item out of the library, leaving a gap. Returning a reference lets the caller read the item without
taking ownership of it.

7. What is the lifetime `'a` in `items_by_author` actually saying?
It says the returned references live as long as self — the library. The references point into the library's own data, so they cannot outlive the library itself.

8. Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the same `Library` at once, and how did you structure the method around that?
Rust only allows one mutable borrow of a value at a time. Both Item and Member live inside Library, so holding two &mut references from the same Library simultaneously is 
not allowed. The solution is to find the indices first using immutable borrows, then use those indices to mutate one at a time.

9. Why are `Library`'s fields private?
Because the library is responsible for keeping the item's LoanStatus and the member's borrowed_item_ids in sync. If fields were public, external code could change one 
without updating the other, leaving the library in an inconsistent state.

10. What duplication does the provided `late_fee_cents` remove, and what would
    you lose by making it a free function instead?
Without it, both MediaKind and Item would each need to implement the same formula: overdue_days * daily_late_fee_cents. The default method in the trait writes that formula 
once and both implementations get it for free. Making it a free function would lose the connection to loan_days() and daily_late_fee_cents() — you'd have to pass those 
values in manually every time.

11. Why is `Result` preferable to `panic!` for validation failures? Name a
    place in this crate where a panic would be defensible.
Result lets the caller decide how to handle a failure — they can recover, retry, or propagate it. panic! crashes the whole program with no recovery. A place where panic is 
defensible is inside library_with_items() in the test helper — if test setup fails, the test is broken and crashing immediately is acceptable.

12. Which derive did you deliberately leave off a type, and why?
Clone was left off Library — cloning a library would create a second copy of all items and members, which could easily lead to the two copies getting out of sync. Since the 
library enforces invariants (item status matching member lists), having two independent copies would be dangerous.

**Experiment A: Compiler Error**
```bash

    Checking rfb_labs_week_2_session_4 v0.1.0 (/home/henry-peters/Desktop/rust-for-bitcoin-2.0/rfb_labs_week_2_session_4)
    error[E0382]: borrow of moved value: `item`
    --> src/main.rs:15:20
    |
    12 |     let item = Item::new(1, "Dune".into(), "Frank Herbert".into(), MediaKind::Book { pages: 320 });
    |         ---- move occurs because `item` has type `Item`, which does not implement the `Copy` trait
    13 |     library.add_item(item);
    |                      ---- value moved here
    14 |
    15 |     println!("{}", item.title);    
    |                    ^^^^^^^^^^ value borrowed here after move
    
    For more information about this error, try `rustc --explain E0382`.
    error: could not compile `rfb_labs_week_2_session_4` (bin "rfb_labs_week_2_session_4") due to 1 previous error
```

**Experiment B error**
```bash
   Compiling rfb_labs_week_2_session_4 v0.1.0 (/home/henry-peters/Desktop/rust-for-bitcoin-2.0/rfb_labs_week_2_session_4)
warning: unused imports: `Item` and `MediaKind`
 --> src/main.rs:3:33
  |
3 | use rfb_labs_week_2_session_4::{Item, Library, LibraryError, MediaKind};
  |                                 ^^^^                         ^^^^^^^^^
  |
  = note: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
  --> src/main.rs:18:5
   |
17 |     let found = library.find_item(1);
   |                 ------- immutable borrow occurs here
18 |     library.checkout(1, 100, 5)?;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
19 |     println!("{:?}", found);
   |                      ----- immutable borrow later used here

For more information about this error, try `rustc --explain E0502`.
warning: `rfb_labs_week_2_session_4` (bin "rfb_labs_week_2_session_4") generated 1 warning
error: could not compile `rfb_labs_week_2_session_4` (bin "rfb_labs_week_2_session_4") due to 1 previous error; 1 warning emitted
```

## Design notes

Describe any choices you made, including how you kept an item's status and its
borrower's list from drifting apart, and (if attempted) the optional generic
search.

## Example output

Paste the output of `cargo run` here once Part 8 is complete.
```bash
warning: `rfb_labs_week_2_session_4` (lib) generated 3 warnings (run `cargo fix --lib -p rfb_labs_week_2_session_4` to apply 3 suggestions)
   Compiling rfb_labs_week_2_session_4 v0.1.0 (/home/henry-peters/Desktop/rust-for-bitcoin-2.0/rfb_labs_week_2_session_4)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.40s
     Running `target/debug/rfb_labs_week_2_session_4`
Checked out: OnLoan { member_id: 100, day_borrowed: 0 }
Late fee: 225 cents
Status after return: Available
Handled error: item 99 not found
```