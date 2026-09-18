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

1. `LoanStatus` is an enum because each item can be in exactly one valid state:
   available, on loan with loan details, or lost. A `bool` plus two `Option`
   fields could accidentally represent nonsense, like "available" but with a
   borrower id.
2. `match` forces every variant to be handled. If a fourth `MediaKind` is added
   later, the compiler points to every place that must decide what that new
   kind means.
3. `Item::new` takes `String`, so the new `Item` owns the title after the call.
4. `add_item` takes `&mut self` because it changes the library, and it takes
   `item` by value because the library becomes the owner of that item.
5. If `add_item` returns `Err`, the passed item was still moved into the
   function and then dropped. That keeps the API simple for this assignment. An
   alternative would be returning `Result<(), (LibraryError, Item)>` so the
   caller can recover the item.
6. `find_item` returns `Option<&Item>` so callers can inspect an item without
   cloning it or taking ownership away from the library.
7. The lifetime `'a` says the returned item references cannot outlive the
   borrowed `Library` they came from.
8. `checkout` cannot hold a `&mut Item` and a `&mut Member` from the same
   `Library` at once because both come from the same owner. I first found their
   indexes, validated using shared reads, then mutated by index after the checks.
9. `Library`'s fields are private so outside code cannot update an item status
   without also updating the member's borrowed list.
10. `late_fee_cents` keeps the overdue-days times daily-fee formula in one
    trait method. As a free function, it would lose the nice `item.late_fee_cents`
    style and would not travel with the `LoanTerms` behavior.
11. `Result` is better than `panic!` for validation because bad ids, duplicate
    ids, and invalid returns are normal caller mistakes that can be handled. A
    panic is defensible in tests when using `unwrap` after setting up data that
    must exist.
12. I deliberately left `Clone` off `Item` and `Member`. The library should own
    them, and lookups should borrow them rather than copying around separate
    versions.

Ownership experiment A:

```text
error[E0382]: borrow of moved value: `item`
  --> examples/ownership_a.rs:13:20
   |
 5 |     let item = Item::new(
   |         ---- move occurs because `item` has type `Item`, which does not implement the `Copy` trait
...
12 |     library.add_item(item)?;
   |                      ---- value moved here
13 |     println!("{}", item.title);
   |                    ^^^^^^^^^^ value borrowed here after move
```

`add_item(item)` takes ownership of `item`, so the caller cannot read
`item.title` afterwards.

Ownership experiment B:

```text
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
  --> examples/ownership_b.rs:15:5
   |
14 |     let held = library.find_item(1).unwrap();
   |                ------- immutable borrow occurs here
15 |     library.checkout(1, 100, 0)?;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
16 |     println!("{held}");
   |                ---- immutable borrow later used here
```

`held` is a reference into `library`. While that reference is still used later,
Rust will not allow `checkout` to mutably borrow and change the same library.

## Design notes

I kept the item status and member borrowed list in sync by validating first and
mutating second in both `checkout` and `return_item`. `checkout` only changes
state after item, member, status, and borrow-limit checks pass. `return_item`
computes the fee first, then marks the item available and removes the id from
the member's list.

I also added the optional `filter_items` helper. `items_by_author` and
`available_items` both use it, so the filtered-search pattern is written once.

## Example output

```text
#1: "The Rust Programming Language" by Steve Klabnik [book (560 pages); on loan to member id 100 since day 10]
late return fee: 225 cents
handled error: item id 99 was not found
```
