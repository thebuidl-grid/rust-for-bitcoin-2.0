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
  
    LoanStatus in an enum because we want to tie different variants i.e Available, OnLoan and Lost
    to the corresponding items. We can therefor be able to handle each item by matching it against this enum.

2. What does `match` force you to do when a fourth `MediaKind` is added later?
    match will force one to handle the added MediaKind in all implementations.

3. `Item::new` takes `String` rather than `&str`. Who owns the title afterwards?
        The Item struct own it afterwards because Item::new() accepts owned string 
        rather than borrwed string &str.

4. Why does `add_item` take `self` by `&mut` but `item` by value?
    add_item changes the state of self and for item we are only reading from it and not changing it's state
    
5. When `add_item` returns `Err`, what happened to the `Item` the caller passed
   in? Was that a good design choice, and what is the alternative?
    The item either had an empty title or it's a duplicate. This was a good choice because the error
    will be propagated to the user showing what happened rather than panicking.

6. Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?
    Because we are reading from the library struct(self) and they can either be an item Some or None(no item)

7. What is the lifetime `'a` in `items_by_author` actually saying?
     'a says the returned Vec<&'a Item> cannot outlive the &'a self borrow of the library it was produced from; the items are borrowed from the library for exactly as long as that particular borrow of self is alive, (the library could outlive this particular borrow once it ends)
8. Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the same
   `Library` at once, and how did you structure the method around that?
    Rust rule says that one can only have one mutable reference at once
    I performed mutation as two separate, non-overlapping iter_mut().find(...) calls in sequence (item status update, then member list update) rather than trying to hold both mutable references at the same time.

9. Why are `Library`'s fields private?

    If items and members were public, any caller could reach in and, say, remove an item from members[x].borrowed_item_ids without also setting that item's status back to Available, silently corrupting the library's internal consistency. Keeping the fields private forces every state change to go through methods like checkout/return_item, which update both sides together — exactly what the doc comment on Library says

10. What duplication does the provided `late_fee_cents` remove, and what would
    you lose by making it a free function instead?

    Making it a default method means the formula exists exactly once, and both impls get it automatically as long as they supply the two smaller building-block methods. As a free function instead, we will lose the automatic availability on every LoanTerms implementor — callers would need to remember to call the free function explicitly rather than it just being "part of the trait," and it wouldn't benefit from trait bounds/generic code written against &dyn LoanTerms or impl LoanTerms

11. Why is `Result` preferable to `panic!` for validation failures? Name a
    place in this crate where a panic would be defensible.

    Validation failures which could be (empty title, duplicate id, unknown item/member, item already on loan, etc.) are expected, ordinary outcomes a caller might trigger through normal use — not bugs. panic! unwinds/aborts the whole program, which is unacceptable behavior for a library whose caller might reasonably want to catch a DuplicateItemId and just try a different id. Result forces the caller to explicitly handle failure via ?/match and decide what to do next, without crashing.
    A defensible place for a panic in this crate: inside checkout's or return_item's internal unwrap() calls where we've already proven the item/member exists earlier in the same function (e.g. the .unwrap() on self.members.iter_mut().find(...) right after we've already confirmed that member id exists via an earlier check) — panicking there will indicate a genuine internal logic bug in the library itself, not a bad caller input, so it's arguably fine as an unwrap()/expect() with a message like "member existence already verified above."

12. Which derive did you deliberately leave off a type, and why?

    Item doesn't derive Clone (or Copy), even though LoanStatus and MediaKind both do. This is deliberate: Part 3 requires find_item, items_by_author, and available_items to return borrowed references "without cloning." If Item derived Clone, it would be easy to sidestep the borrowing exercise entirely by just cloning items out of the library instead of learning to work with &Item. Leaving Clone off forces the borrowing discipline the assignment is testing.

**Experiment A** — reading `item_3.title` after `library.add_item(item_3)?`:

```text
error[E0382]: borrow of moved value: `item_3`
  --> src/main.rs:47:16
   |
44 | let item_3 =
   |     ------ move occurs because `item_3` has type `Item`, which does not implement the `Copy` trait
45 |  Item::new(7, "btcacademy".into(), "Jeremiah".into(), MediaKind::Book { pages: 255 });
46 | library.add_item(item_3)?;
   |                  ------ value moved here
47 | println!("{}", item_3.title); //  triggers the error
   |                ^^^^^^^^^^^^ value borrowed here after move
For more information about this error, try `rustc --explain E0382`.
```

`add_item(&mut self, item: Item)` takes `Item` by value. Since `Item` derives
neither `Copy` nor `Clone`, `library.add_item(item_3)` moves `item_3`'s data
into the library's internal `Vec<Item>`, leaving the `item_3` binding with no
data. The next line tries to read `item_3.title` after that move, and the
compiler rejects the use-after-move at compile time.


**Experiment B** — holding `library.find_item(6)`'s result across a mutable `checkout` call:

```text
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
  --> src/main.rs:50:1
   |
49 | let found = library.find_item(6);
   |             ------- immutable borrow occurs here
50 | library.checkout(6, 1, 1)?;
   | ^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
51 | println!("{}", found.unwrap().title); // triggers the error
   |                ----- immutable borrow later used here
For more information about this error, try `rustc --explain E0502`.
```

`find_item(&self, ...) -> Option<&Item>` borrows from `library` for as long as
`found` is alive. `checkout(&mut self, ...)` needs an exclusive mutable borrow
of `library`, but `found` is still in scope and used on the next line. Rust
won't allow a mutable borrow to coexist with a live immutable borrow of the
same value, since `checkout` could mutate or reallocate the underlying data
that `found` still points into, leaving `found` dangling. The compiler
rejects this at compile time instead.

## Design notes

Describe any choices you made, including how you kept an item's status and its
borrower's list from drifting apart, and (if attempted) the optional generic
search.

  checkout and return_item are the only two places allowed to change an item's LoanStatus or a member's borrowed_item_ids, and both always update the two together in the same call rather than leaving one to be set separately. Since Rust won't allow holding &mut Item and &mut Member at once through the same Library, each method performs the two mutations as separate, sequential iter_mut().find(...) calls — item status first, then the member's list — instead of trying to borrow both simultaneously. Both methods also validate everything first and mutate only after every check passes, so a rejected checkout or return_item never leaves the library half-updated. Keeping Library's fields private is what makes this enforceable: callers can only reach items and members through methods that keep both sides in sync, never by mutating either field directly.
  I attempted part 9 and  filter_items takes any Fn(&Item) -> bool predicate and returns matching borrowed references; items_by_author and available_items are now both expressed as one-line calls to it with different closures, commented the duplicate iterate-filter-collect pattern that existed between them before.

## Example output

```
┌──(julypjulius㉿kali)-[~/bitcoin-lightning-network/rust-for-bitcoin-2.0/rfb_labs_week_2_session_4]
└─$ cargo r
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running `target/debug/rfb_labs_week_2_session_4`
Fee owed after return: 75
Checked out item 5
Handled Err: Error: Item of id: 999, was not found
```

Fee owed after return: 75 — item 6 was kept 24 days (checked out day 1, returned day 25), 3 days over its 21-day limit, at 25¢/day = 75 cents.
Checked out item 5 — the first match succeeded, so it printed the Ok arm.
Handled Err: Error: Item of id: 999, was not found — the second match tried checking out a nonexistent item, caught the error, and printed it via Display instead of crashing — this satisfies Part 8's "print one handled error" requirement.

