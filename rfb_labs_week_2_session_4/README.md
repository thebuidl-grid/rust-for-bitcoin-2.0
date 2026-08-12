# Rust for Bitcoin 2.0 — Week 2, Session 4

Build a small lending library while practising structs, enums, traits,
ownership, borrowing, collections, and `Result`-based error handling. No
Bitcoin and no external crates — just Rust.

The crate is intentionally incomplete. Search for `TODO` and implement each
part; do not change the public type names or function signatures.

## Recommended workflow

**Explain why `LoanStatus` is an enum rather than a `bool` plus two `Option` fields, and what `match` forces you to handle.**
- Modeling LoanStatus as an enum instead of combining booleans with optional fields enforces the idiomatic Rust principle of making unrepresentable states impossible. A struct with flags like is_on_loan alongside optional member_id and day_borrowed values invites bug-prone edge cases—such as an item being marked as borrowed while lacking a member ID, or simultaneously being flagged as both active and lost. By leveraging an enum with algebraic variants (Available, OnLoan { member_id, day_borrowed }, and Lost), the data structure guarantees structural correctness at compile time. Associated payload fields exist strictly when relevant, eliminating runtime null checks and ensuring zero-cost, memory-efficient state management.

- Beyond state guarantees, using an enum unlocks Rust’s strict exhaustiveness checking through match expressions. The compiler forces your code to handle every possible state explicitly, meaning if you later introduce a new variant like InTransit, the build will safely fail until every corresponding control path accounts for it. Furthermore, pattern matching enables safe structural unpacking of variant data, guaranteeing you can only access borrowing metadata when an item is actually checked out. By shifting runtime state validation entirely into the type system, you get cleaner, self-documenting code with zero risk of invalid data access.


1. Read [ASSIGNMENT.md](ASSIGNMENT.md).
2. Complete Part 2 in `error.rs`, then Part 3 in `library.rs`.
3. Remove `#[ignore]` from the relevant test and run it.
4. Complete the traits in Part 4 and the two operations in Parts 5–6.
5. Run the ownership experiments and record the errors.
6. Build the demo in `main.rs`.
7. Add the remaining required tests yourself.

**experiment A**
```bash
error[E0382]: borrow of moved value: `item`                                                                                    
  --> src\main.rs:26:20
   |
18 |     let item = Item::new(
   |         ---- move occurs because `item` has type `Item`, which does not implement the `Copy` trait
...
24 |     library.add_item(item)?;
   |                      ---- value moved here
25 |
26 |     println!("{}", item.title);
   |                    ^^^^^^^^^^ value borrowed here after move
```
**experiment B**
```bash
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
  --> src\main.rs:34:5
   |
33 |     let item_ref = library.find_item(1);
   |                    ------- immutable borrow occurs here
34 |     library.checkout(1, 101, 1)?;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
35 |
36 |    println!("{:?}", item_ref);
   |                     -------- immutable borrow later used here
```

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
- An enum enforces invalid states to be unrepresentable. Using a bool alongside Option<u32> fields allows illegal combinations—such as is_loaned = false while holding a borrower ID and due date. The OnLoan enum variant safely packages borrower details together only when the item is actually on loan.

2. What does `match` force you to do when a fourth `MediaKind` is added later?
- Rust’s match expressions enforce exhaustive pattern matching. If a fourth MediaKind variant is added, the Rust compiler will flag every unhandled match across the codebase as a compile error, preventing unhandled cases at runtime.
3. `Item::new` takes `String` rather than `&str`. Who owns the title afterwards?
- The created Item struct owns the title String. The caller gives up ownership of its string when passing it by value into Item::new.
4. Why does `add_item` take `self` by `&mut` but `item` by value?
- &mut self: The library needs to modify its internal collections (e.g., pushing into a Vec), which requires mutable access without taking full ownership of the library.

- item by value: The library needs full ownership of the Item so it can store and manage it in its internal collection for the item's entire lifetime 
5. When `add_item` returns `Err`, what happened to the `Item` the caller passed
   in? Was that a good design choice, and what is the alternative?
- What happened is that the Item passed by value was dropped at the end of add_item when the function returned early with an Err, losing the item data forever.

- It is a poor design choice because the caller loses their data on failure.

- Alternative: Return the item back inside the error variant—e.g., Result<(), (LibraryError, Item)> or Err(LibraryError::DuplicateItemId { item })—so the caller regains ownership.

6. Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?
- find_item returns Option<&Item> (a reference) because the caller only wants to inspect or query the item, not take ownership. Returning Option<Item> would attempt to move the item out of the Library container, which Rust's borrow checker prevents.

7. What is the lifetime `'a` in `items_by_author` actually saying?
- The lifetime 'a explicitly connects the output to the input: it guarantees that the returned Vec<&'a Item> references will remain valid for as long as the borrowed Library (&'a self) lives.
8. Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the same
   `Library` at once, and how did you structure the method around that?
-  Holding &mut Item and &mut Member simultaneously requires two mutable borrows from the same Library struct, violating Rust’s aliasing XOR mutability rule (E0502 / E0499).
- Workaround structure: Look up items and members separately using their indices/IDs, compute validate checks, update the member's borrowed set, and then mutate the item's LoanStatus. Alternatively, perform state checks first before taking mutable references briefly one after another.
9. Why are `Library`'s fields private?
- Encapsulation. Private fields protect domain invariants—such as preventing an item from marked as Available while a member still has its ID in their borrowed list. Modifying fields must go through public method boundaries.
10. What duplication does the provided `late_fee_cents` remove, and what would
    you lose by making it a free function instead?
- removing duplication Centralizes the late-fee computation logic across different MediaKind variants so the library return workflow doesn't re-implement loan period / rate arithmetic.
- i  would lose object-oriented ergonomics and clean encapsulation—callers could no longer write item.late_fee_cents(...) or utilize trait-based polymorphism if late_fee_cents becomes part of a trait later.
11. Why is `Result` preferable to `panic!` for validation failures? Name a
    place in this crate where a panic would be defensible.
- Result represents expected, recoverable failures (e.g., member limit reached, item not found) and forces callers to explicitly handle errors. A panic! should be reserved for unrecoverable internal invariant violations or corrupted states (e.g., an internal index out of bounds that indicates a bug in the code).
12. Which derive did you deliberately leave off a type, and why?
- Copy was left off types like Item and Member because they hold heap-allocated owned strings (String) and dynamic collections (Vec). Implementing Copy on non-bitwise-copyable types is forbidden by Rust; forcing intentional Clone operations avoids accidental expensive allocations.

## Design notes

Describe any choices you made, including how you kept an item's status and its
borrower's list from drifting apart, and (if attempted) the optional generic
search.
- Preventing Data Drift (Status vs. Member List): To keep Item::status and Member::borrowed_items in sync, all mutations are strictly encapsulated behind checkout and return_item. Neither items nor members collections are exposed directly. On checkout, both the item status update and member ID insertion are performed inside the same method, ensuring an atomic-like operation.

- Part 9 - Generic Search: Implemented a higher-order method filter_items<P>(&self, predicate: P) -> Vec<&Item> where P: Fn(&Item) -> bool. By delegating items_by_author and available_items to filter_items, we removed duplicated .iter().filter(...).collect() boilerplate while preserving lifetime elision.

## Example output
```bash
--- Library Stocked & Member Registered ---

--- Borrowing Item ---
[1] "Dune" by Frank Herbert (Book (320 pages)) - On loan to member 100 since day 1

--- Returning Item (Late) ---
Returned item 1 on day 30. Late fee owed: 200 cents ($2.00)

--- Handling an Error ---
Handled expected error: item 1 cannot be returned because it is not currently on loan 
```

Paste the output of `cargo run` here once Part 8 is complete.
