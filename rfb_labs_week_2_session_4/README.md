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

Part A

```text

❯ cargo check
    Checking rfb_labs_week_2_session_4 v0.1.0 (/Users/hakeem/Desktop/26/rust-for-bitcoin-2.0/rfb_labs_week_2_session_4)
error[E0382]: borrow of moved value: `item`
  --> src/main.rs:19:20
   |
11 |     let item = Item::new(
   |         ---- move occurs because `item` has type `Item`, which does not implement the `Copy` trait
...
18 |     library.add_item(item)?;
   |                      ---- value moved here
19 |     println!("{}", item.title);
   |                    ^^^^^^^^^^ value borrowed here after move

For more information about this error, try `rustc --explain E0382`.
error: could not compile `rfb_labs_week_2_session_4` (bin "rfb_labs_week_2_session_4") due to 1 previous error
```

This error happens because `item.title` is trying to be accessed after `library.add_item(item)` has been called which moved item out of this scope so `item` is no longer accessible

Part B

```text
❯ cargo check
    Checking rfb_labs_week_2_session_4 v0.1.0 (/Users/hakeem/Desktop/26/rust-for-bitcoin-2.0/rfb_labs_week_2_session_4)
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
  --> src/main.rs:24:5
   |
23 |     let found = library.find_item(1);
   |                 ------- immutable borrow occurs here
24 |     library.checkout(1, 500, 5)?;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
25 |     println!("{:?}", found);
   |                      ----- immutable borrow later used here

For more information about this error, try `rustc --explain E0502`.
error: could not compile `rfb_labs_week_2_session_4` (bin "rfb_labs_week_2_session_4") due to 1 previous error
```

This error occurs because checkout takes a mutable borrow and found already has an immutable borrow, so calling found later on in the println macro makes this impossible because what if the value gets updated in checkout, this is why Rust enforces the rule that no mutable and immutable borrow should exist in the same scope at the same time.

1. Why is `LoanStatus` an enum rather than a `bool` plus two `Option` fields?

`LoanStatus` is an enum because only one of its values can be derived to be true in this case the loan status of a book can either be Available, on loan to a member, or lost.

2. What does `match` force you to do when a fourth `MediaKind` is added later?

`match` will force you to cover all possibilities, it is exhaustive. Adding a fourth `MediaKind` later just means I will have to cover for that extra field in my `match`.

3. `Item::new` takes `String` rather than `&str`. Who owns the title afterwards?

the title is owned by `Item` because `title: String` parameter is taken by value, then moved into the function and then moved again into the title field of the new `Item` struct. So whoever owns the `Item` which is eventually the `Library` by `self.items.push(item)` owns the String too

4. Why does `add_item` take `self` by `&mut` but `item` by value?

`add_item` takes `self` by `&mut` but `item` by value because `self` is still called outside the `add_item` function so it needs to be a reference to it that will be passed to `add_item` and it is `mut` because self is getting updated with an `item`. Which is also why it takes `item` by value, because it is being moved into the library items.

5. When `add_item` returns `Err`, what happened to the `Item` the caller passed
   in? Was that a good design choice, and what is the alternative?

The `Item` the caller passed in gets dropped when `add_item` returns `Err`, I think it was a good design choice maybe, and an alternative would be to return `Item` back to the caller like refusing to take it in because adding it produces an error so it is returned back to the caller.

6. Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?

`find_item` returns `Option<&Item>` because the function only performs a read operation to `Item` so the value does not have to be returned and only a reference is enough.

7. What is the lifetime `'a` in `items_by_author` actually saying?

The lifetime signature is basically saying the references inside the returned `Vec` cannot outlive the borrow of `self` that produced them, which is guaranteeing memory safety without an actual owner change.

8. Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the same
   `Library` at once, and how did you structure the method around that?

checkout cannot hold `&mut Item` and `&mut Member` from the same `Library` at once because Item data could be present in `Member`, specifically `id` in `Item` could be a part of `borrowed_item_ids` so it cannot be allowed to hold a mutable borrow of both of them since one of them could potentially update a value which could cause an error when the other tries to point to that value.

9. Why are `Library`'s fields private?

Because `Library` is responsible for keeping an item's `LoanStatus` and a member's borrowed-id list in agreement.

10. What duplication does the provided `late_fee_cents` remove, and what would
    you lose by making it a free function instead?

`late_fee_cents` being implemented in `LoanTerms` removes duplication by the implementations of `LoanTerms` which are `MediaKind` and `Item` and making it a free function would lose its being not duplicated since both implementations would have the same function and the fact that it is related to `LoanTerms` which is why it was defined under it will be lost as well.

11. Why is `Result` preferable to `panic!` for validation failures? Name a
    place in this crate where a panic would be defensible.

Because `panic!` exits the program and that is undesirable for validation failures and reserved for more ideally more core bugs in the program, so `Result` is perfect for handling validation failures.

12. Which derive did you deliberately leave off a type, and why?
    For `Item` and `Member`, `Clone` is not derive because `Item` is not cheap to duplicate and it would be trivial to accidentally clone an item out of the library, mutate the clone's status, and have it drift silently out of sync with the real `Item` sitting in `self.items`, so leaving `Clone` off forces every part of the codebase to work with borrowed `&Item` references back into the single source of truth, rather than making independent copies that could get out of sync. Same reasoning applies to `Member`

## Design notes

Describe any choices you made, including how you kept an item's status and its
borrower's list from drifting apart, and (if attempted) the optional generic
search.

- `checkout` and `return_item` follow a validate-then-mutate order, every error path returns early via `?` or `return Err(...)` before any field is touched. This guarantees I don't end up with half-applied change, e.g an item marked `OnLoan` but the member's list not updated.

- `checkout` and `return_item` look up items/members by id inside the method rather than accepting a `&mut Item` or `&mut Member` from the caller, which was demonstrated in Experiment B how we can't hold a borrow into self across a call that needs &mut self.

- the `filter_items<F: Fn(&Item) -> bool>` centralizes the "walk `self.items`, keep the ones matching a predicate" logic that `items_by_author` and `available_items` previously duplicated. Each now just supplies a different closure so the iteration logic exists just once

## Example output

Paste the output of `cargo run` here once Part 8 is complete.

❯ cargo run
Compiling rfb_labs_week_2_session_4 v0.1.0 (/Users/hakeem/Desktop/26/rust-for-bitcoin-2.0/rfb_labs_week_2_session_4)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.85s
Running `target/debug/rfb_labs_week_2_session_4`
item has an empty title
