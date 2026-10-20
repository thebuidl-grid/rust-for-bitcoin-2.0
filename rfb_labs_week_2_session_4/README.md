# Rust for Bitcoin 2.0 — Week 2, Session 4

Build a small lending library while practising structs, enums, traits, ownership, borrowing, collections, and `Result`-based error handling. No Bitcoin and no external crates — just Rust.

The crate is intentionally incomplete. Search for `TODO` and implement each part; do not change the public type names or function signatures.

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

`cargo test` checks the starter project. Ignored tests intentionally exercise unfinished code; enable them progressively rather than leaving them ignored in the submission.

## Written answers

Answer in your own words. Add both ownership compiler errors from Part 7 as fenced text blocks, then explain what caused each.

1. Why is `LoanStatus` an enum rather than a `bool` plus two `Option` fields?

Enums enforce state invariants at compile time. A boolean combined with options creates illegal state combinations (e.g., is_loaned = false with borrower_id = Some(100) or day_borrowed = Some(5)). An enum (Available, OnLoan { member_id, day_borrowed }, Lost) makes invalid states unrepresentable.

2. What does `match` force you to do when a fourth `MediaKind` is added later?

Pattern matching in Rust is exhaustive. Adding a new variant (e.g., Video) triggers compiler errors across every match expression on MediaKind until the new variant is explicitly handled, preventing missing branch bugs.

3. `Item::new` takes `String` rather than `&str`. Who owns the title afterwards?

The newly constructed Item struct takes complete ownership of the String. Taking String avoids complex lifetime annotations and prevents internal cloning.

4. Why does `add_item` take `self` by `&mut` but `item` by value?

self requires &mut because Library mutates its internal state by pushing to items: Vec<Item>. item is taken by value because Library assumes full ownership of the item to store it inside the collection.

5. When `add_item` returns `Err`, what happened to the `Item` the caller passed in? Was that a good design choice, and what is the alternative?

The Item was moved into add_item and dropped when the function scope ended on error, consuming the value. While simple, the caller loses the item. An alternative design is returning Result<(), (LibraryError, Item)>, yielding ownership of the item back to the caller upon validation failure.

6. Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?

It borrows a view of the item without moving it out of Library or cloning heap-allocated fields (String).

7. What is the lifetime `'a` in `items_by_author` actually saying?

It specifies that the returned item references (Vec<&'a Item>) remain valid for as long as the borrowed reference to Library (&'a self) remains active.

8. Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the same `Library` at once, and how did you structure the method around that?

Rust strictly forbids simultaneous mutable references into the same collection (self.items / self.members). To structure around this, validation checks use immutable borrows (&self) first. Once validation succeeds, short-lived sequential iter_mut calls perform state updates safely.

9. Why are `Library`'s fields private?

Encapsulation guarantees that internal state invariants—keeping an Item's LoanStatus in sync with a Member's borrowed_item_ids list—cannot be corrupted or desynchronized by external callers.

10. What duplication does the provided `late_fee_cents` remove, and what would you lose by making it a free function instead?

It centralizes the shared fee calculation (days_held - loan_days) * daily_rate into a single trait default method so implementing types (MediaKind, Item) do not repeat arithmetic. Making it a free function would lose ergonomic method-chaining syntax (item.late_fee_cents(days)).

11. Why is `Result` preferable to `panic!` for validation failures? Name a place in this crate where a panic would be defensible.

Validation failures (e.g., unknown ID, borrow limit reached) are expected domain outcomes that callers must handle gracefully. A panic crashes the entire process. A panic is defensible during internal invariant checks or unit test setup where failure indicates a logic bug rather than invalid runtime input.

12. Which derive did you deliberately leave off a type, and why?

Copy was left off Item and Member because they hold heap-allocated fields (String, Vec<u32>). Types managing dynamic heap memory cannot implement Copy.


# Part 7 — Ownership Experiments

**Experiment A: Reading item.title after add_item**

```rust
let item = Item::new(1, "Dune".into(), "Frank Herbert".into(), MediaKind::Book { pages: 320 });
library.add_item(item)?;
println!("{}", item.title);
```

**Compiler Error (cargo check):**

```
error[E0382]: borrow of moved value: `item`
  --> src/main.rs:15:20
   |
13 |     let item = Item::new(1, "Dune".into(), "Frank Herbert".into(), MediaKind::Book { pages: 320 });
   |         ---- move occurs because `item` has type `Item`, which does not implement the `Copy` trait
14 |     library.add_item(item)?;
   |                      ---- value moved here
15 |     println!("{}", item.title);
   |                    ^^^^^^^^^^ value borrowed here after move
```

**Explanation:**

add_item takes item by value, transferring ownership into the Library. Accessing item.title afterwards fails because item is no longer valid in the caller's scope.

**Experiment B: Holding reference across mutable checkout call**

```rust
let item_ref = library.find_item(1).unwrap();
library.checkout(1, 100, 5)?;
println!("{}", item_ref.title);
```

**Compiler Error (cargo check):**

```
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
  --> src/main.rs:18:5
   |
17 |     let item_ref = library.find_item(1).unwrap();
   |                    ------- immutable borrow occurs here
18 |     library.checkout(1, 100, 5)?;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
19 |     println!("{}", item_ref.title);
   |                    -------------- immutable borrow later used here
```

**Explanation:**

find_item creates an immutable borrow of library. Calling checkout requires a mutable borrow (&mut library). Rust forbids active mutable borrows while an immutable borrow (item_ref) is still in scope.
## Design notes

Describe any choices you made, including how you kept an item's status and its borrower's list from drifting apart, and (if attempted) the optional generic search.

- **State Synchronization:** In checkout and return_item, all checks are completed before mutating state. Updates to Item::status and Member::borrowed_item_ids happen atomically in sequential steps within the same method, maintaining consistent cross-references.

- **Validation Ordering:** checkout validates in strict order: unknown item, unknown member, lost item, item already on loan, then borrow limit reached.

## Example output

Paste the output of `cargo run` here once Part 8 is complete.
```
elsuraj@El-suraj:~/rust-for-bitcoin-2.0/rfb_labs_week_2_session_4$ cargo run 
   Compiling rfb_labs_week_2_session_4 v0.1.0 (/home/elsuraj/rust-for-bitcoin-2.0/rfb_labs_week_2_session_4)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 37.75s
     Running `target/debug/rfb_labs_week_2_session_4`
--- Checking out Item 1 on day 10 ---
[1] "Mastering Bitcoin" by Andreas Antonopoulos (Book (398 pages)) - On loan to member 101 since day 10

--- Returning Item 1 on day 40 ---
Returned item 1. Late fee owed: 225 cents

--- Demonstrating Handled Error ---
Handled error: Member ID 999 not found
elsuraj@El-suraj:~/rust-for-bitcoin-2.0/rfb_labs_week_2_session_4$ 
```