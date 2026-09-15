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

1. **Why is `LoanStatus` an enum rather than a `bool` plus two `Option`
   fields?**
   An item can only ever be in exactly one of three states —
   `Available`, `OnLoan { member_id, day_borrowed }`, or `Lost` — and the
   `OnLoan` data (`member_id`, `day_borrowed`) only makes sense together and
   only while on loan. A `bool` plus two separate `Option<u32>` fields can
   represent invalid combinations the type system would happily allow, such
   as "not on loan" but with a `Some(member_id)` left over, or "on loan" with
   `day_borrowed` still `None`. The enum makes the invalid states
   unrepresentable: there is no way to construct an `OnLoan` without both
   pieces of data, and no way to have loan data while `Available`.

2. **What does `match` force you to do when a fourth `MediaKind` is added
   later?**
   Every non-wildcard `match` over `MediaKind` (in `loan_days`,
   `daily_late_fee_cents`, and `Display`) stops compiling until the new
   variant is handled. The compiler's exhaustiveness check turns "did I
   remember every call site that cares about media kind?" from a manual
   audit into a build failure, so a forgotten case is caught at compile time
   instead of surfacing as a bug at runtime.

3. **`Item::new` takes `String` rather than `&str`. Who owns the title
   afterwards?**
   The `Item` owns the title. Passing a `String` (rather than borrowing
   `&str`) moves the caller's string into the constructor, and `Item::new`
   moves it again into the `title` field of the `Item` it returns. There is
   no borrow to outlive, and no lifetime parameter needed on `Item`.

4. **Why does `add_item` take `self` by `&mut` but `item` by value?**
   `add_item` needs to mutate the library's `items` vector (push into it),
   so it needs `&mut self` — a shared reference wouldn't allow that. `item`
   is taken by value because the library is meant to own every item it
   stocks; taking it by reference would leave ownership with the caller,
   which contradicts "the library owns everything," and would need the item
   to outlive the library or be cloned back out later.

5. **When `add_item` returns `Err`, what happened to the `Item` the caller
   passed in? Was that a good design choice, and what is the alternative?**
   The `Item` is dropped. Because `add_item(item: Item)` takes ownership
   unconditionally, the value is gone by the time the function can decide
   whether to accept it — on the rejected paths it is simply never pushed
   into `self.items`, so it goes out of scope at the end of the call and its
   memory is freed. This is a reasonable choice for this assignment (the
   caller is not expected to retry with the exact same `Item` — an empty
   title or a duplicate id isn't something you fix and resubmit unchanged),
   but it does mean a caller who wants to reuse the value after a failed
   insert can't. The alternative is to return the rejected item back inside
   the error, e.g. `Err((LibraryError, Item))`, or to take `&Item` and clone
   internally only once validation passes — at the cost of a needless clone
   on the success path.

6. **Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?**
   `Library` owns its items in a `Vec<Item>`; returning `Option<Item>` would
   require moving (or cloning) the item out of that vector, which either
   breaks the library's invariant that it owns every item it stocks, or
   forces an unnecessary clone on every lookup. Returning a borrowed
   `Option<&Item>` lets the caller read the item without taking ownership,
   and the borrow checker ensures the reference cannot outlive the `Library`
   it points into.

7. **What is the lifetime `'a` in `items_by_author` actually saying?**
   `pub fn items_by_author<'a>(&'a self, author: &str) -> Vec<&'a Item>` says
   that every reference in the returned `Vec` borrows from `self` for
   exactly as long as the `&'a self` borrow is alive — the references in the
   result cannot outlive the `Library` they point into. It says nothing
   about `author`'s lifetime, since `author` is only read during the call
   and nothing derived from it is returned.

8. **Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the
   same `Library` at once, and how did you structure the method around
   that?**
   Both the item and the member live inside fields of the same `Library`, so
   two simultaneous `&mut` borrows into `self` — one reached via
   `self.items` and one via `self.members` — would each need `&mut self` (or
   overlapping sub-borrows the borrow checker can't statically prove are
   disjoint through two separate `iter_mut().find(...)` calls at once). Rust
   only allows one live mutable borrow of a value at a time, so I structured
   `checkout` in two passes: first, an immutable-borrow validation pass that
   looks up the item and member by id and returns early with an appropriate
   `Err` for every unknown-id, lost-item, already-on-loan, and
   over-the-limit case (those immutable borrows end as soon as the `match`
   /`if` finishes); second, once every check has passed, a mutation pass
   that re-borrows the item mutably, sets its new `LoanStatus`, then
   re-borrows the member mutably and pushes the item id onto their
   `borrowed_item_ids`. The two mutable borrows never overlap because the
   first one's `.unwrap()` and use is fully finished (and dropped) before
   the second begins.

9. **Why are `Library`'s fields private?**
   So `Library` is the only code that can put `items` and `members` in an
   inconsistent state. If `items` and `members` were public, any caller
   could set an item's status to `Available` while it's still listed in a
   member's `borrowed_item_ids`, or push an item with a duplicate id
   straight into the vector, bypassing every check in `add_item` and
   `checkout`. Keeping the fields private forces every mutation through
   methods that can enforce the item/member invariants together.

10. **What duplication does the provided `late_fee_cents` remove, and what
    would you lose by making it a free function instead?**
    Without the default method, both `impl LoanTerms for MediaKind` and
    `impl LoanTerms for Item` would need to repeat the same
    `saturating_sub` + multiply formula, and any future third implementor of
    `LoanTerms` would need to repeat it again. Writing it once as a default
    trait method means every implementor gets the fee formula for free just
    by supplying `loan_days` and `daily_late_fee_cents`. A free function
    (e.g. `fn late_fee_cents(loan_days: u32, daily_fee: u32, days_held: u32)
    -> u32`) would compute the same number, but it wouldn't be attached to
    the trait — callers couldn't write generic code like
    `fn charge<T: LoanTerms>(x: &T, days: u32) -> u32 { x.late_fee_cents(days)
    }`, and nothing would stop a future implementor of `LoanTerms` from
    forgetting to call it or reimplementing the formula slightly wrong.

11. **Why is `Result` preferable to `panic!` for validation failures? Name a
    place in this crate where a panic would be defensible.**
    Validation failures here — an unknown id, a duplicate id, a return day
    before the borrow day — are ordinary, expected outcomes of normal use
    (a caller mistyped an id, or two callers raced), not programmer bugs.
    `Result` lets the caller decide how to respond (retry, surface a message
    to a librarian, log and move on) without unwinding or crashing the whole
    program. `panic!` is for states that should be *impossible* if the
    code is correct. A defensible panic in this crate is the
    `.unwrap()` calls in `checkout`/`return_item` on the second,
    mutation-pass lookup of an item/member id that the first, validation
    pass already confirmed exists — if that lookup ever failed it would mean
    `Library`'s own invariants were broken by a bug in this crate, not by
    anything a caller did.

12. **Which derive did you deliberately leave off a type, and why?**
    `MediaKind`, `LoanStatus`, and `Item` all derive `PartialEq`/`Eq` (used
    throughout the tests via `assert_eq!`) but none of them derive
    `Ord`/`PartialOrd`. There's no single sensible total ordering for an
    `Item` — you might want to order by title, by author, by id, or by loan
    length depending on context — so leaving `Ord` off forces call sites
    that want a specific ordering to say so explicitly (e.g.
    `items.sort_by_key(|item| &item.title)` or the `max_by_key(|item|
    item.loan_days())` used in `longest_loan_item`), rather than silently
    picking a field order that happens to fall out of derive.

## The two experiments

Run each, paste the real `cargo check` error into `README.md`, explain it,
then comment the line out. **A** — read `item.title` after
`library.add_item(item)?`. **B** — hold the result of `library.find_item(1)`,
call `library.checkout(..)?`, then print what you held.

### Experiment A

```text
error[E0382]: borrow of moved value: `item`
 --> examples/experiment_a.rs:7:20
  |
5 |     let item = Item::new(1, "Dune".into(), "Frank Herbert".into(), MediaKind::Book { pages: 412 });
  |         ---- move occurs because `item` has type `Item`, which does not implement the `Copy` trait
6 |     library.add_item(item)?;
  |                      ---- value moved here
7 |     println!("{}", item.title);
  |                    ^^^^^^^^^^ value borrowed here after move
```

`add_item` takes `item: Item` by value, so calling `library.add_item(item)?`
moves `item` into the function. `Item` doesn't implement `Copy` (it owns
heap-allocated `String`s), so after the call the local binding `item` no
longer owns any data — the compiler considers it moved-from and refuses to
let `item.title` borrow from it. This is exactly the invariant from written
answer 3/4: once ownership of the title has moved into the `Library`, the
original binding can no longer be used to reach it.

### Experiment B

```text
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
  --> examples/experiment_b.rs:9:5
   |
 8 |     let found = library.find_item(1);
   |                 ------- immutable borrow occurs here
 9 |     library.checkout(1, 100, 0)?;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
10 |     println!("{:?}", found);
   |                      ----- immutable borrow later used here
```

`found` holds a `&Item` borrowed from `library` (via `find_item(&self, ..)`).
That immutable borrow is still alive at the `println!` on the last line, so
it's also considered alive across the `library.checkout(..)` call in
between. `checkout` needs `&mut self`, and Rust never allows a mutable borrow
to coexist with a live immutable borrow of the same value — if it did,
`checkout` could reallocate `self.items` (via `Vec` growth) or otherwise
invalidate the exact `&Item` that `found` points at, leaving `found` a
dangling reference. The fix is to use `found` (or drop it) before calling
`checkout`, or to re-borrow with `find_item` again after the mutation.

## Testing checklist

Write tests for a successful checkout, an item that cannot be lent twice, the
borrow limit, a late return's fee, an on-time return owing nothing, an ebook
returned late still owing nothing, and author search returning borrowed
items. Also test each validation error. The repository contains a few ignored
starter tests; remove their `#[ignore]` attributes and add the remaining
cases.

All starter tests are un-ignored and passing, and `tests/library.rs` adds
coverage for: an on-time return, a late ebook return, checking out an item
already on loan, each `add_item`/`register_member`/`checkout`/`return_item`
validation error (`EmptyTitle`, `DuplicateItemId`, `DuplicateMemberId`,
`ItemNotFound`, `MemberNotFound`, `ItemIsLost`, `ItemNotOnLoan`,
`InvalidReturnDay`), in addition to the provided
`BorrowLimitReached`/checkout/late-fee/author-search cases. Run with:

```bash
cargo test
```

## Design notes

- **Keeping status and the borrower list in sync:** `checkout` and
  `return_item` are the only two places that ever write `LoanStatus::OnLoan`
  / `LoanStatus::Available` or push/remove from a member's
  `borrowed_item_ids`, and each does both updates together in the same
  method, after all validation has already passed (see written answer 8 for
  why validation and mutation are split into two passes). Because
  `Library`'s fields are private, no other code can update one without the
  other, so the two can't drift apart.
- **Validate first, mutate second:** both `checkout` and `return_item` do all
  their fallible lookups and checks against immutable borrows first, and
  only start mutating once every check has passed (using `.unwrap()` on the
  second-pass lookups, which are known-safe because the first pass already
  proved the ids exist). This both satisfies the borrow checker (only one
  mutable borrow, and only once needed) and keeps `Library` from ending up
  half-updated if a later check were to fail.
- **`late_fee_cents` as a default trait method:** `MediaKind` and `Item` both
  implement `LoanTerms` by supplying `loan_days`/`daily_late_fee_cents`
  only; the shared `overdue_days * daily_late_fee_cents` formula lives once,
  as the trait's default `late_fee_cents` (see written answer 10).
- **Part 9 — generic search:** implemented as `Library::filter_items<F: Fn(&Item)
  -> bool>(&self, predicate: F) -> Vec<&Item>`. Both `items_by_author` and
  `available_items` are now one-line calls into it
  (`self.filter_items(|item| item.author == author)` and
  `self.filter_items(|item| item.status == LoanStatus::Available)`), so the
  actual "walk every item and collect matching references" logic exists in
  exactly one place.

## Example output

```text
Ada returned item 2 on time, owing 0 cents.
Ada returned item 1 late, owing 225 cents.
[1] "Dune" by Frank Herbert — book (412 pages) — available
[2] "Project Hail Mary" by Andy Weir — audiobook (970 minutes) — available
[3] "The Rust Programming Language" by Steve Klabnik — ebook (1200 KB) — available
handled error: no item with id 999 was found
```
