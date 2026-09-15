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

### Part 7 experiments

**Experiment A** — reading `item.title` after `library.add_item(item)?`:

```
error[E0382]: borrow of moved value: `item`
  --> src/main.rs:58:20
   |
56 |     let item = Item::new(5, "Experiment".into(), "Author".into(), MediaKind::Book { pages: 1 });
   |         ---- move occurs because `item` has type `Item`, which does not implement the `Copy` trait
57 |     library.add_item(item)?;
   |                      ---- value moved here
58 |     println!("{}", item.title);
   |                    ^^^^^^^^^^ value borrowed here after move
```

`add_item` takes `item: Item` by value, not by reference, so the call moves
ownership of the `Item` into the library's `Vec<Item>`. Once that happens the
`item` binding in `main` no longer owns any data — it's not a stale pointer,
it's simply gone, and the compiler statically forbids reading through it.
`Item` doesn't derive `Copy` (it owns a heap-allocated `String` for `title`
and `author`), so there's no implicit duplicate made at the call site the way
there would be for an `i32`.

**Experiment B** — holding `find_item(1)` across a `checkout` call:

```
error[E0502]: cannot borrow `*library` as mutable because it is also borrowed as immutable
  --> src/main.rs:69:5
   |
68 |     let held = library.find_item(1);
   |                ------- immutable borrow occurs here
69 |     library.checkout(1, 100, 0)?;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
70 |     println!("{held:?}");
   |                ---- immutable borrow later used here
```

`find_item` returns `Option<&Item>`, an immutable borrow tied to `library`'s
lifetime. Because `held` is read again in the `println!` after `checkout`,
the borrow checker has to keep that immutable borrow alive across the
`checkout(&mut self, ...)` call — and Rust's aliasing rule is that a value
can have any number of immutable (`&`) borrows *or* exactly one mutable
(`&mut`) borrow, never both at the same time. This is the compiler catching
a real bug: if it allowed this, `checkout` could reallocate `library.items`
(e.g. on a `Vec` growth) while `held` still pointed at the old memory,
leaving `held` dangling.

### Answers

1. **Why is `LoanStatus` an enum rather than a `bool` plus two `Option`
   fields?** An enum makes the three states (`Available`, `OnLoan { .. }`,
   `Lost`) mutually exclusive by construction. A `bool` (`is_on_loan`) plus
   `Option<u32>` (`borrower`) plus `Option<u32>` (`day_borrowed`) would let
   you represent nonsense combinations, like `is_on_loan = false` while
   `borrower = Some(100)`, or a `borrower` with no `day_borrowed`. The enum
   makes invalid states impossible to construct instead of something you
   have to remember to keep in sync.

2. **What does `match` force you to do when a fourth `MediaKind` is added
   later?** Every `match` on `MediaKind` in this crate (`loan_days`,
   `daily_late_fee_cents`, `Display`) is written without a `_` catch-all
   arm, so it's exhaustive. Adding a fourth variant (say `Magazine`) makes
   the compiler refuse to build until every one of those `match` sites
   handles the new variant explicitly. It turns "did I remember to update
   every place that cares about media kinds?" from a manual audit into a
   compile error.

3. **`Item::new` takes `String` rather than `&str`. Who owns the title
   afterwards?** The `Item` itself owns the `String` — `Item::new` takes the
   `String` by value and stores it in `self.title`. The caller that built the
   string (e.g. via `.into()` on a `&str` literal) gives up ownership at the
   call site; they can't use that original binding afterward unless they
   cloned it first.

4. **Why does `add_item` take `self` by `&mut` but `item` by value?**
   `&mut self` borrows the `Library` just long enough to push into its
   `Vec<Item>` — the caller keeps their `Library` afterward, they've only
   lent it out temporarily. `item` is taken by value because the whole point
   is a permanent transfer: the `Library` becomes the sole owner of that
   `Item` going forward, and nothing else should be able to mutate it out
   from under the library's bookkeeping.

5. **When `add_item` returns `Err`, what happened to the `Item` the caller
   passed in? Was that a good design choice, and what is the alternative?**
   Because `item` was moved into `add_item`, it's dropped at the end of the
   function on the `Err` path — the caller's data is gone even though the
   operation failed. That's a real cost of this design: a rejected item (say,
   for an empty title) can't be inspected or fixed and retried without the
   caller having built it again from scratch. The alternative is returning
   the item back inside the error, e.g. `Result<(), (LibraryError, Item)>`
   or a dedicated error variant that carries the item, so a failed call is
   recoverable. This crate chose the simpler signature since the two
   `add_item` failure cases (empty title, duplicate id) are things the
   caller already knows how to avoid before calling.

6. **Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?**
   Returning `Option<Item>` would require moving (or cloning) the item out of
   `self.items`, which either breaks the `Vec`'s invariants (you can't move a
   value out of a `Vec` you don't own end-to-end) or silently duplicates data
   the library is supposed to be the single source of truth for. A borrowed
   `&Item` lets the caller read the item without taking ownership, and the
   compiler ties the reference's lifetime to `&self`, so it can't outlive the
   `Library` it points into.

7. **What is the lifetime `'a` in `items_by_author` actually saying?**
   `pub fn items_by_author<'a>(&'a self, author: &str) -> Vec<&'a Item>` says
   that every reference in the returned `Vec` is only valid for as long as
   the `&self` borrow used to produce it stays valid. In other words: "the
   items I hand back point into this library, so you can't keep using them
   after (or while mutably borrowing) the library in a way that would
   invalidate them." It's what lets the borrow checker catch experiment B —
   the compiler already knows any reference derived from `&self` is tied to
   that borrow's lifetime.

8. **Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the
   same `Library` at once, and how did you structure the method around
   that?** Rust only allows one `&mut` borrow of a given value at a time,
   and both the item and the member live inside the same `Library` — so
   getting a `&mut Item` from `self.items` and a `&mut Member` from
   `self.members` "at once" through the same `&mut self` isn't something the
   borrow checker will let you hold simultaneously if it can't prove the two
   borrows are disjoint through a single access path. Rather than fight
   that, `checkout` is split into two phases: first it *validates* using
   only immutable lookups (`find_item`, `find_member`), returning early on
   any `Err` before anything is touched; only once every check has passed
   does it *mutate*, and even then it does so one borrow at a time — it
   looks up and updates the `Item`'s status, lets that mutable borrow end,
   then separately looks up and updates the `Member`'s `borrowed_item_ids`.
   The two mutable borrows never overlap.

9. **Why are `Library`'s fields private?** So that `items` and `members` can
   only change through the methods `Library` provides. If `items` were
   public, any caller could push a duplicate-id `Item` directly into the
   `Vec`, or flip an `Item`'s `status` to `Available` without also clearing
   it from the corresponding `Member`'s `borrowed_item_ids` — silently
   breaking the invariant that an item's status and its borrower's list
   always agree. Privacy forces every mutation through `checkout` /
   `return_item` / `add_item`, which is where that invariant is enforced.

10. **What duplication does the provided `late_fee_cents` remove, and what
    would you lose by making it a free function instead?** Both `MediaKind`
    and `Item` implement `LoanTerms`, and both need the exact same formula —
    `days_held.saturating_sub(loan_days()) * daily_late_fee_cents()`. Putting
    it in the trait as a default method means that formula is written once
    and both implementers get it automatically by only supplying
    `loan_days` and `daily_late_fee_cents`. A free function
    (`fn late_fee_cents(loan_days: u32, daily_fee: u32, days_held: u32) -> u32`)
    would remove the *arithmetic* duplication but not the *call-site*
    duplication — every caller would need to know to fetch `loan_days()` and
    `daily_late_fee_cents()` first and pass them in by hand, and the formula
    would no longer be discoverable as "part of what a `LoanTerms` type can
    do" via autocomplete/method syntax (`item.late_fee_cents(days_held)`).

11. **Why is `Result` preferable to `panic!` for validation failures? Name a
    place in this crate where a panic would be defensible.** A `panic!`
    unwinds (or aborts) the whole program — there's no way for a caller to
    catch it, log it, show the user a message, and keep going. Every failure
    in this crate (empty title, unknown id, borrow limit, etc.) is an
    ordinary, expected outcome of normal use — a caller checking out a book
    that's already on loan isn't a bug, it's just something they need to be
    told about — so `Result` lets that be handled like any other value. A
    panic would be defensible for a genuine *programmer* error rather than
    bad input: for example, the `.expect("checked above")` calls inside
    `checkout`/`return_item`, right after this crate itself just verified
    the id exists a few lines earlier. If that lookup ever failed there, it
    would mean the library's own logic is inconsistent, not that the caller
    did anything wrong — exactly the kind of bug a panic should surface loudly
    during development rather than quietly turning into a wrong `Result`.

12. **Which derive did you deliberately leave off a type, and why?** Neither
    `Item` nor `Member` derives `Clone` (or `Copy`). Both own data (`title`/
    `author` strings, a `Vec<u32>` of borrowed ids) that the `Library` is
    supposed to be the single owner and single source of truth for. If
    `Item` were `Clone`, a caller could clone the item they got back from
    `find_item`, mutate the clone's `status` locally, and now have a copy
    that's silently out of sync with the real one still sitting in the
    library — exactly the kind of drift `checkout`/`return_item` exist to
    prevent. Leaving `Clone` off makes that class of bug impossible instead
    of just discouraged.

## Design notes

The core invariant this crate protects is: **an item's `LoanStatus` and its
borrower's `borrowed_item_ids` must always agree.** If item 1 is
`OnLoan { member_id: 100, .. }`, then member 100's list must contain `1`, and
vice versa. Two things keep that true:

- `Library`'s fields are private, so the only way to change either side of
  that relationship is through `checkout` and `return_item` — there's no
  path that lets a caller update one half without the other.
- Both of those methods follow the same shape: **validate everything first
  using immutable borrows, and only mutate once every check has passed.**
  `checkout` updates the item's status and pushes onto the member's list in
  the same successful call; `return_item` resets the status to `Available`
  and removes the id from the member's list in the same successful call.
  There's no intermediate state where a caller could observe the item
  updated but the member not yet (or vice versa) — either the whole checkout
  succeeds and both sides move together, or it returns an `Err` and neither
  side is touched at all.

I also implemented the optional Part 9 generic search. `filter_items<F: Fn(&Item) -> bool>`
does the actual `iter().filter().collect()` work once, and `items_by_author`
and `available_items` are now both one-line wrappers that just supply a
different predicate:

```rust
pub fn items_by_author<'a>(&'a self, author: &str) -> Vec<&'a Item> {
    self.filter_items(|item| item.author == author)
}

pub fn available_items(&self) -> Vec<&Item> {
    self.filter_items(|item| item.status == LoanStatus::Available)
}
```

## Example output

```
Ada returned "Dune" on time and owes 0 cents.
Ada returned "Project Hail Mary" late and owes 500 cents.
Handled error: no item with id 999
```
