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

1. **Why is `LoanStatus` an enum rather than a `bool` plus two `Option` fields?**
   The enum makes the three states mutually exclusive by construction; a bool plus
   two Options permits impossible combos the compiler can't catch, and `match`
   forces you to handle every state.

2. **What does `match` force you to do when a fourth `MediaKind` is added later?**
   Every `match` on it stops compiling until you add an arm (and give `LoanTerms`
   the new kind's numbers), so no use site is silently ignored.

3. **`Item::new` takes `String` rather than `&str`. Who owns the title afterward?**
   The `Item` does; once `add_item` moves it in, the `Library` owns it. The caller
   loses it unless it cloned first.

4. **Why does `add_item` take `self` by `&mut` but `item` by value?**
   `&mut self` to mutate the vec; by value so the library becomes the sole owner
   and stores it — a `&Item` would just borrow, forcing copies.

5. **When `add_item` returns `Err`, what happened to the `Item` the caller passed
   in? Was that a good design choice, and what is the alternative?**
   It's dropped — moved in, rejected, deallocated. Acceptable in a small crate;
   the friendlier alternative returns it: `Result<(), (LibraryError, Item)>`.

6. **Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?**
   `Item` isn't `Copy`; returning by value means cloning or moving it out of the
   vec. A reference borrows the canonical stored copy for free.

7. **What is the lifetime `'a` in `items_by_author` actually saying?**
   The returned refs live exactly as long as the `&self` borrow — they point into
   the library and can't outlive it. It's elidable since `&self` pins the output.

8. **Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the same
   `Library` at once, and how did you structure the method around that?**
   Both were obtained through methods borrowing all of `self`. I validate with
   `&self` reads first, drop those borrows, then mutate via `iter_mut()` on each
   field separately so no two mutable borrows overlap.

9. **Why are `Library`'s fields private?**
   So callers can't mutate an item's `LoanStatus` or a member's list directly and
   desync them; the library alone keeps the two in agreement.

10. **What duplication does the provided `late_fee_cents` remove, and what would
    you lose by making it a free function instead?**
    The shared "overdue days × fee, 0 if on time" formula. As a free fn you'd pass
    `loan_days`/`daily_fee` by hand, losing type-directed dispatch, per-type
    override, and the "ebook never late" case.

11. **Why is `Result` preferable to `panic!` for validation failures? Name a
    place in this crate where a panic would be defensible.**
    Panics kill the program; validation failures are expected callers should
    handle and test. A panic is defensible when a *stored invariant* is corrupted
    — a genuine bug, not a caller mistake.

12. **Which derive did you deliberately leave off a type, and why?**
    `Clone`/`Copy` on `Item` and `Member` (they own `String`/`Vec`; `Copy` is
    impossible and `Clone` invites accidental copies where the assignment wants
    borrowing). `MediaKind`/`LoanStatus` do derive `Copy`.

## Design notes

Both operations follow "validate first, mutate second", so a failing call never
leaves partial state. The item's `LoanStatus` is the single source of truth:
`checkout` sets `OnLoan { member_id, .. }` and pushes the id onto *that same*
member's list; `return_item` reads `member_id` from the status, sets it back to
`Available`, and prunes with `retain`. Mutating one side without the other isn't
expressible inside the methods, and the private fields keep callers from doing
it behind the library's back.

Part 9 was attempted: `filter_items(predicate: impl Fn(&Item) -> bool)` makes
`items_by_author` and `available_items` one-liners delegating to it, removing
the duplicated filter loops.

## Example output

```text
late fee: 150 cents
handled: Item with id 9999 not found
```


## Ownership Experiment Result 

```bash

test@pop-os:~/Desktop/rust/rust-for-bitcoin-2.0/rfb_labs_week_2_session_4$ cargo check
    Checking rfb_labs_week_2_session_4 v0.1.0 (/home/test/Desktop/rust/rust-for-bitcoin-2.0/rfb_labs_week_2_session_4)
error[E0382]: borrow of moved value: `items`
   --> src/main.rs:39:28
    |
 10 |     let items = vec![
    |         ----- move occurs because `items` has type `Vec<Item>`, which does not implement the `Copy` trait
...
 34 |     for item in items {
    |                 ----- `items` moved due to this implicit call to `.into_iter()`
...
 39 |     let first_item_title = items[0].title.clone();
    |                            ^^^^^ value borrowed here after move
    |
note: `into_iter` takes ownership of the receiver `self`, which moves `items`
   --> /home/test/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/iter/traits/collect.rs:312:18
    |
312 |     fn into_iter(self) -> Self::IntoIter;
    |                  ^^^^
help: consider iterating over a slice of the `Vec<Item>`'s content to avoid moving into the `for` loop
    |
 34 |     for item in &items {
    |                 +

warning: unused variable: `first_item_title`
  --> src/main.rs:39:9
   |
39 |     let first_item_title = items[0].title.clone();
   |         ^^^^^^^^^^^^^^^^ help: if this is intentional, prefix it with an underscore: `_first_item_title`
   |
   = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

For more information about this error, try `rustc --explain E0382`.
warning: `rfb_labs_week_2_session_4` (bin "rfb_labs_week_2_session_4") generated 1 warning
error: could not compile `rfb_labs_week_2_session_4` (bin "rfb_labs_week_2_session_4") due to 1 previous error; 1 warning emitted
```
