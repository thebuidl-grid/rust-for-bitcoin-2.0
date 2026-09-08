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

### Experiment A — reading `item.title` after `library.add_item(item)?`

```
error[E0382]: borrow of moved value: `item`
  --> src/main.rs:60:32
   |
53 |     let item = Item::new(
   |         ---- move occurs because `item` has type `Item`, which does not implement the `Copy` trait
...
59 |     library.add_item(item)?;
   |                      ---- value moved here
60 |     println!("still have: {}", item.title);
   |                                ^^^^^^^^^^ value borrowed here after move
```

`add_item` takes `item: Item` by value, so calling it moves the `Item` into
the function — and from there into the library's `Vec<Item>`. `Item` derives
neither `Copy` nor `Clone`, so nothing is left behind in the caller's local
variable `item`. The compiler tracks this at the type level: after the move,
`item` is no longer a valid binding, and any further use — even a read-only
field access like `item.title` — is a compile error rather than a runtime
bug. This is the whole point of taking `Item` by value in `add_item`: the
library becomes the sole owner of the item, so nobody can accidentally hold a
stale, disconnected copy of a title while the real item's status changes
underneath them.

### Experiment B — holding `library.find_item(1)` across a `checkout` call

```
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
  --> src/main.rs:76:5
   |
75 |     let held = library.find_item(1);
   |                ------- immutable borrow occurs here
76 |     library.checkout(1, 100, 0)?;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
77 |     println!("still have: {:?}", held);
   |                                  ---- immutable borrow later used here
```

`find_item` returns `Option<&Item>`, a borrow tied to `library`'s lifetime.
`held` keeps that borrow alive because it's used again on the line after
`checkout`. `checkout` needs `&mut self` to update an item's status, and Rust
never allows a mutable borrow to coexist with a live immutable one — if it
did, `checkout` could move or invalidate the very `Item` that `held` points
at, leaving `held` dangling. The fix is to shrink the immutable borrow's
scope: use `held` (or drop it) before calling `checkout`, or re-borrow with a
fresh `find_item` call after the mutation.

1. **Why is `LoanStatus` an enum rather than a `bool` plus two `Option`
   fields?** A `bool` (`is_on_loan`) plus two `Option` fields (say,
   `Option<u32>` for the borrower and `Option<u32>` for the day borrowed)
   can represent illegal states that have no real-world meaning: `is_on_loan
   == false` with `Some(member_id)` still set, or `is_on_loan == true` with
   `borrower == None`. Every one of those combinations has to be treated as
   "shouldn't happen" and defended against by convention, not by the type
   system. `LoanStatus` collapses "on loan" and "who/when" into one
   variant, `OnLoan { member_id, day_borrowed }`, so the borrower and the
   borrow day only exist when the item actually is on loan — there is no
   way to construct the inconsistent states above. `match` also forces every
   caller to handle `Available`, `OnLoan`, and `Lost` explicitly (or add a
   deliberate `_` catch-all); with the `bool`-plus-`Option` design, it's easy
   to check the `bool` and forget the `Lost` case doesn't exist yet, or to
   unwrap an `Option` that the reader assumed was always `Some`.
2. **What does `match` force you to do when a fourth `MediaKind` is added
   later?** Every non-wildcard `match` on `MediaKind` (in `loan_days`,
   `daily_late_fee_cents`, and `Display`) stops compiling until the new
   variant is given an arm. The compiler enumerates every call site that
   needs an opinion about the new kind, so it's impossible to ship a build
   where, say, `loan_days` silently falls through for the new variant. A
   `_` wildcard arm would defeat this — none of the `match`es in this crate
   use one for `MediaKind`, on purpose.
3. **`Item::new` takes `String` rather than `&str`. Who owns the title
   afterwards?** `Item::new` takes ownership of the `String` and moves it
   into the `Item`'s `title` field, so the `Item` owns its title for as long
   as it exists. The caller passes in an owned `String` (or converts a
   `&str` with `.into()`/`.to_string()`) and gives it up; there's no
   borrowing relationship to a caller-owned buffer to keep alive.
4. **Why does `add_item` take `self` by `&mut` but `item` by value?**
   `add_item` mutates the library's internal `Vec<Item>` (it pushes onto it),
   so it needs `&mut self` to be allowed to change `Library`'s state at all.
   `item`, on the other hand, is meant to become part of that state
   permanently — the library needs to *own* it, not just borrow it for the
   duration of the call — so it's taken by value and moved into the vector.
5. **When `add_item` returns `Err`, what happened to the `Item` the caller
   passed in? Was that a good design choice, and what is the alternative?**
   On the `Err` path in this implementation, the `Item` is simply dropped —
   it was moved into `add_item`'s parameter, the validation fails before it
   is pushed anywhere, and the function returns without ever giving it back.
   The caller's data is gone. That's a reasonable choice for a lab exercise
   where the rejected cases (an empty title, a duplicate id) are usually
   caller bugs rather than something to retry with the same value, but it's
   not free: a caller who *did* want to fix the title and retry has to
   reconstruct the `Item` from scratch. The alternative is to return the
   rejected value alongside the error, e.g. `Err((LibraryError, Item))` or a
   dedicated error variant that carries the item back, so the caller can
   recover it without rebuilding it.
6. **Why does `find_item` return `Option<&Item>` rather than
   `Option<Item>`?** Returning `Option<Item>` would require cloning (or
   moving, which isn't possible — the library still needs the item) every
   time anyone looked something up, and `Item` doesn't derive `Clone`
   precisely because there should only ever be one of each item. A borrowed
   `Option<&Item>` lets callers read the item's current fields without
   copying anything or taking the library's data away from it; the library
   remains the single owner.
7. **What is the lifetime `'a` in `items_by_author` actually saying?** The
   signature is `fn items_by_author<'a>(&'a self, author: &str) -> Vec<&'a
   Item>`. `'a` ties the lifetime of every reference in the returned `Vec`
   to the lifetime of the `&self` borrow used to call the method. In plain
   terms: the returned references are only valid as long as the borrow of
   `library` that produced them is still alive — the compiler will not let
   the caller hold onto the `Vec<&Item>` past the point where `library` is
   mutated or dropped. Note `author: &str` deliberately has its own,
   unrelated lifetime — the returned items don't borrow from the search
   string, only from the library.
8. **Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the
   same `Library` at once, and how did you structure the method around
   that?** `checkout` only has one `&mut self` where `self` is the whole
   `Library`, and Rust's borrow checker doesn't (in general) know that
   `items` and `members` are disjoint fields once you try to hold two
   long-lived mutable borrows produced by separate method calls or held
   across other mutations — and more importantly, both the item and the
   member need to be validated together before *either* is mutated, so
   holding two live `&mut` references while still reading from both isn't
   what's wanted anyway. Instead, `checkout` first finds the *indices* of
   the item and the member with `Vec::position` (an immutable operation),
   runs every validation check by reading through those indices, and only
   after every check has passed does it index into `self.items` and
   `self.members` to mutate each one — each indexing expression is a short,
   independent, non-overlapping mutable borrow rather than one held across
   the whole function.
9. **Why are `Library`'s fields private?** Because `Library` is responsible
   for an invariant that spans both fields: an item's `LoanStatus` and the
   borrower's `borrowed_item_ids` must always agree (an item is `OnLoan {
   member_id, .. }` if and only if that member's list contains the item's
   id). If `items` and `members` were public, any caller could push to one
   vector, mutate an item's `status` directly, or otherwise change one side
   of that relationship without updating the other, silently corrupting the
   library's state. Keeping the fields private forces every state change to
   go through methods like `checkout` and `return_item`, which update both
   sides together.
10. **What duplication does the provided `late_fee_cents` remove, and what
    would you lose by making it a free function instead?** Without it, both
    `impl LoanTerms for MediaKind` and `impl LoanTerms for Item` would need
    their own copy of "subtract the free loan period from the days held,
    floor at zero, multiply by the daily rate" — two places that could drift
    out of sync if someone tweaked the formula in only one. Because
    `late_fee_cents` is a default method on the trait, defined once in terms
    of `loan_days()` and `daily_late_fee_cents()`, every type that
    implements `LoanTerms` gets the formula for free and only has to supply
    the two small facts that vary per type. A free function taking `&dyn
    LoanTerms` (or generic over `T: LoanTerms`) could achieve the same
    de-duplication, but it would be a function you have to know to call
    instead of a method that comes attached to the type — `item.late_fee_cents(days)`
    reads naturally at the call site, and it can't accidentally be called
    with a `loan_days`/`daily_late_fee_cents` pair that don't belong to the
    same type, the way passing the wrong values to a free function could.
11. **Why is `Result` preferable to `panic!` for validation failures? Name a
    place in this crate where a panic would be defensible.** The failures
    this crate validates for — an unknown id, a duplicate id, a borrow limit,
    a bad return day — are all *expected*, recoverable outcomes of normal use
    of a library by outside callers; a caller passing an id that doesn't
    exist yet is not a bug in this crate, it's routine input that deserves a
    value the caller can inspect, match on, and recover from. `panic!`
    unwinds (or aborts) the whole program and gives the caller no chance to
    do anything but crash, which is far too heavy a hammer for "that item id
    isn't registered." A panic is defensible only where the failure would
    mean this crate's own invariant was broken by a bug inside the crate
    itself, not by caller input — which is exactly the `.expect(...)` in
    `return_item` (`src/library.rs`), where the member id on an active loan
    is looked up after `checkout` is the *only* code path that ever sets
    that id, and `checkout` never sets it to an unregistered member. If that
    lookup ever failed, it would mean `Library`'s own invariant was already
    broken elsewhere — a bug worth crashing loudly on, not a `Result` for a
    caller to route around.
12. **Which derive did you deliberately leave off a type, and why?** `Item`
    derives `Debug` and `PartialEq, Eq` but not `Clone` (or `Copy`). Adding
    `Clone` would make it easy to accidentally produce a second `Item` with
    the same id that drifts out of sync with the one the library actually
    owns and mutates through `checkout`/`return_item` — exactly the
    "single owner" property Question 6 relies on. Leaving `Clone` off is
    also what makes Experiment A a compile error instead of a silent bug:
    without `Clone`, there's no way to keep using the value after
    `add_item` takes ownership of it.

## Design notes

`Library`'s two collections (`items: Vec<Item>` and `members: Vec<Member>`)
are private, and every mutation that touches the item/member relationship —
`checkout` and `return_item` — goes through a single method that updates both
sides in the same call. `checkout` looks up both the item's and the member's
*index* first (immutable operations), runs every validation against those
indices, and only mutates `self.items[item_index].status` and
`self.members[member_index].borrowed_item_ids` after every check has passed —
so a validation failure can never leave one side updated and the other not.
`return_item` mirrors this: it reads the loan's `member_id`/`day_borrowed` out
of the item's `LoanStatus`, computes the fee, resets the item to `Available`,
and then removes the id from that same member's list, all before returning.
Because both fields only ever change together, inside these two methods,
there's no path through the public API that can produce an item marked
`OnLoan` whose borrower's list doesn't mention it (or vice versa).

The shared fee math lives once, as a default method on the `LoanTerms` trait
(`late_fee_cents`), built from the two facts each implementor supplies
(`loan_days`, `daily_late_fee_cents`). `Item` delegates both to its `kind`,
so the loan-term rules are defined exactly once, on `MediaKind`, and reused
everywhere an `Item`'s terms are needed (`checkout`'s implicit note about
which day the item is due back, `return_item`'s fee calculation, and
`longest_loan_item`).

For the optional Part 9, `filter_items<F: Fn(&Item) -> bool>` was added to
`Library`, and both `items_by_author` and `available_items` are now one-line
calls into it (`self.filter_items(|item| item.author == author)` and
`self.filter_items(|item| item.status == LoanStatus::Available)`), rather
than each writing its own `iter().filter(...).collect()`.

## Example output

```
checked out: #1 "Dune" by Frank Herbert (book, 412 pages) — on loan to member 100 since day 0
returned on day 10, fee owed: 0 cents
item after return: #1 "Dune" by Frank Herbert (book, 412 pages) — available
returned late on day 20, fee owed: 150 cents
handled error: no member with id 999
```
