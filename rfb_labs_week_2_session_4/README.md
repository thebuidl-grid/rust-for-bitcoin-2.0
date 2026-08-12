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

## Ownership experiments (Part 7)

**Experiment A** — read `item.title` after `library.add_item(item)?`:

```
error[E0382]: borrow of moved value: `experiment_item`
  --> src/main.rs:36:20
   |
29 |     let experiment_item = Item::new(
   |         --------------- move occurs because `experiment_item` has type `Item`, which does not implement the `Copy` trait
...
35 |     library.add_item(experiment_item)?;
   |                      --------------- value moved here
36 |     println!("{}", experiment_item.title);
   |                    ^^^^^^^^^^^^^^^^^^^^^ value borrowed here after move
```

`add_item` takes `item: Item` by value, not by reference, so calling it moves
`experiment_item` into the library — the library is now the sole owner of
that `Item`. `experiment_item` is no longer a valid binding in `main` after
the call, so reading `.title` off it is a use of moved data, caught at
compile time rather than becoming a dangling read at runtime.

**Experiment B** — hold the result of `library.find_item(1)`, call
`library.checkout(..)?`, then print what was held:

```
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
  --> src/main.rs:45:5
   |
44 |     let held = library.find_item(1);
   |                ------- immutable borrow occurs here
45 |     library.checkout(1, 7, 5)?;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
46 |     println!("{:?}", held);
   |                      ---- immutable borrow later used here
```

`find_item` returns `Option<&Item>`, a shared borrow tied to `library`'s
lifetime. `checkout` needs `&mut self`, and Rust will not allow a mutable
borrow to start while a shared borrow of the same value is still alive (i.e.
still used later, per `held`'s use on the `println!` line). This is exactly
the hazard the borrow checker exists to prevent: if `checkout` were allowed
to run while `held` still pointed at item 1, that reference could be
observing a half-updated `LoanStatus` — the checker refuses to compile it
instead of letting it become a runtime data race or a stale read.

Both lines are commented out in `src/main.rs` (search for "Experiment A" and
"Experiment B") so the crate builds; the surrounding code that produced each
error is left in place.

## Written answers

1. **Why is `LoanStatus` an enum rather than a `bool` plus two `Option`
   fields?** An enum makes invalid combinations impossible to represent. A
   `bool` (`on_loan`) plus two `Option` fields (`member_id`, `day_borrowed`)
   would allow states like `on_loan: false` with `member_id: Some(7)`, which
   means nothing — the compiler can't stop you from constructing it, and
   every reader has to reason about whether the fields agree. `LoanStatus`
   ties the borrower id and borrow day to the `OnLoan` variant itself, so
   there is exactly one active state at a time and no way to build a
   contradictory one.

2. **What does `match` force you to do when a fourth `MediaKind` is added
   later?** Every non-wildcard `match` over `MediaKind` (in `loan_days`,
   `daily_late_fee_cents`, and the `Display` impl) stops compiling until a
   new arm is added for the new variant. The compiler finds every call site
   that needs updating instead of the new variant silently falling through
   to whatever the last matching arm happened to do.

3. **`Item::new` takes `String` rather than `&str`. Who owns the title
   afterwards?** The `Item` owns the title — the `String` passed in is moved
   into the struct's `title` field. Because `Item` owns its data outright, it
   needs no lifetime parameter and can be stored in `Library`'s `Vec<Item>`
   for as long as the library exists, independent of whatever string the
   caller originally had.

4. **Why does `add_item` take `self` by `&mut` but `item` by value?**
   `&mut self` because the method only needs to mutate the `Library` in
   place (push onto `self.items`) — it doesn't need to consume or replace
   the `Library` itself. `item` by value because the library must become the
   permanent owner of that `Item` (it lives in the `Vec` from now on); taking
   it as value transfers that ownership directly instead of requiring a
   clone or tying `Item` to a borrowed lifetime.

5. **When `add_item` returns `Err`, what happened to the `Item` the caller
   passed in? Was that a good design choice, and what is the alternative?**
   The `Item` was moved into `add_item`'s parameter, and since the `Err`
   branch never pushes it into `self.items`, it is simply dropped at the end
   of the function — the caller has lost it. That's a reasonable choice for
   a small assignment (the API stays simple: `Result<(), LibraryError>`), but
   it is mildly unfriendly to a caller who wants to fix one field and retry
   without rebuilding the whole `Item`. The alternative is to hand the value
   back on failure, e.g. `Result<(), (LibraryError, Item)>` or a dedicated
   error variant that wraps the rejected `Item`, so the caller can recover
   and retry.

6. **Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?**
   Returning a reference lets the caller look at the item without taking it
   away from the library. `Item` isn't `Clone`, so `Option<Item>` would have
   to either move the item out of the `Vec` (leaving a hole, and breaking
   `Library`'s job of owning every item) or not compile at all. A borrow is
   free and keeps `Library` as the single owner.

7. **What is the lifetime `'a` in `items_by_author` actually saying?** It
   says the returned `Vec<&'a Item>` may not outlive the `&'a self` borrow it
   came from — every reference in the vector is only valid for as long as
   that particular borrow of the library is alive. It's the compiler's way
   of guaranteeing the caller can't hang onto those references after, say,
   the library is dropped or mutably borrowed again.

8. **Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the
   same `Library` at once, and how did you structure the method around
   that?** If both were obtained through methods that each take `&mut self`
   (e.g. two calls to a hypothetical `find_item_mut(&mut self, ...)` and
   `find_member_mut(&mut self, ...)`), the two mutable borrows of `self`
   would overlap, and the borrow checker rejects two live `&mut` borrows of
   the same value. I avoided this by borrowing the two different *fields*
   directly inside `checkout` — `self.items.iter_mut().find(...)` and
   `self.members.iter_mut().find(...)` — which Rust's field-sensitive borrow
   checking treats as disjoint borrows of `self.items` and `self.members`,
   not of `self` as a whole, so both can be held mutably at the same time.
   Validation happens first (using `.is_none()` checks and reading
   `.status`), and only once every rule has passed do I mutate the item's
   status and push onto the member's `borrowed_item_ids` in the same block.

9. **Why are `Library`'s fields private?** So that `checkout` and
   `return_item` are the only ways to change an item's `LoanStatus` or a
   member's `borrowed_item_ids`, and both methods update the two together.
   If `items` and `members` were public, outside code could set an item to
   `OnLoan` without touching the member's list (or vice versa), and the two
   would drift out of agreement with nothing to stop it.

10. **What duplication does the provided `late_fee_cents` remove, and what
    would you lose by making it a free function instead?** It removes the
    need for both `impl LoanTerms for MediaKind` and `impl LoanTerms for
    Item` to repeat the "daily rate × days held" formula — each only has to
    supply `daily_late_fee_cents`, and the shared default method handles the
    arithmetic once. A free function could compute the same number, but
    every caller would have to fetch `daily_late_fee_cents()` themselves and
    pass it in by hand, and you'd lose the ability to call
    `.late_fee_cents(days)` polymorphically through the trait (e.g. on a
    `&dyn LoanTerms` or a generic `T: LoanTerms`) or to override it for a
    specific type later without touching every call site.

11. **Why is `Result` preferable to `panic!` for validation failures? Name a
    place in this crate where a panic would be defensible.** `Result`
    encodes an expected, recoverable failure in the function's signature, so
    the compiler forces every caller to acknowledge it (handle it or
    propagate it with `?`) instead of the program silently crashing on
    ordinary bad input like an unknown id. `panic!` is for states that
    should be impossible if the library's own code is correct — a caller of
    the public API should never be able to trigger one just by passing bad
    data. In this crate, the `.unwrap()` calls inside `checkout` and
    `return_item` on the item/member lookups (immediately after the
    corresponding `.is_none()` checks return early) are exactly that: if one
    ever panicked, it would mean the two internal borrows had gotten out of
    sync with each other, a bug in the library itself, not something a
    caller did.

12. **Which derive did you deliberately leave off a type, and why?** `Item`
    (and `Member`) do not derive `Clone`/`Copy`, unlike `MediaKind` and
    `LoanStatus`, which are small, self-contained values and do derive
    `Copy`. `Library` is meant to be the single owner of each `Item`, and
    `checkout`/`return_item` rely on there being exactly one copy of an
    item's `LoanStatus` at a time. Making `Item` cheaply cloneable would
    make it easy to accidentally create a second copy of an item with the
    same id but a diverging status — clone it before checkout, and now one
    copy says `Available` while the real one in the library says `OnLoan`.

## Design notes

`Library` keeps an item's `LoanStatus` and its borrower's `borrowed_item_ids`
from drifting apart by making both fields private and only ever changing them
together, inside `checkout` and `return_item`. `checkout` validates every
rule first (unknown item, unknown member, lost, already on loan, borrow
limit) using read-only lookups, and only mutates the item's status and pushes
onto the member's list after every check has passed — so a failed checkout
never leaves either side partially updated. `return_item` mirrors this: it
validates the item's state and the return day with `checked_sub` before
touching anything, then sets the item back to `Available` and removes the id
from the member's list in the same successful branch. Because there is no
public way to reach `items`/`members` except through `Library`'s methods,
this invariant can't be broken from outside the module.

I did attempt the optional generic search (Part 9): `filter_items<F: Fn(&Item)
-> bool>` takes an arbitrary predicate and returns `Vec<&Item>`, and both
`items_by_author` and `available_items` are now expressed as one-line calls
into it (`filter_items(|item| item.author == author)` and
`filter_items(|item| item.status == LoanStatus::Available)`) rather than each
writing their own `.iter().filter(...)` loop.

## Example output

```
Item 1: Item {
    id: 1,
    title: "Things Fall Apart",
    author: "Chinua Achebe",
    kind: Book {
        pages: 1000,
    },
    status: Available,
}
On-time return fee: 0 cents
Late return fee: 225 cents
Handled error: ItemAlreadyOnLoan: The item id: 2 has being loaned to member id: 7
```
