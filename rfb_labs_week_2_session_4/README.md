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

### The two ownership experiments

**Experiment A** — reading `item.title` after `library.add_item(item)?`:

```
error[E0382]: borrow of moved value: `item`
  --> src/main.rs:60:20
   |
58 |     let item = Item::new(5, "Foundation".into(), "Isaac Asimov".into(), MediaKind::Book { pages: 255 });
   |         ---- move occurs because `item` has type `Item`, which does not implement the `Copy` trait
59 |     library.add_item(item)?;
   |                      ---- value moved here
60 |     println!("{}", item.title);
   |                    ^^^^^^^^^^ value borrowed here after move
```

`add_item` takes `item: Item` by value, so the call at line 59 moves `item`
into the function. `Item` holds `String` fields and doesn't derive `Copy`, so
there is no implicit duplication — after the move the local binding `item` is
no longer valid, and reading `item.title` on the next line tries to use a
value that has already been given away.

**Experiment B** — holding the result of `library.find_item(1)`, calling
`library.checkout(..)?`, then printing what was held:

```
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
  --> src/main.rs:67:5
   |
66 |     let found = library.find_item(1);
   |                 ------- immutable borrow occurs here
67 |     library.checkout(2, 100, 0)?;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
68 |     println!("{}", found.unwrap());
   |                    ----- immutable borrow later used here
```

`find_item` returns `Option<&Item>`, an immutable borrow of `library` that
lives as long as `found` is still used. `checkout` needs `&mut self` to update
the item and member. The borrow checker won't allow a mutable borrow to start
while an immutable borrow that is still going to be read later is alive —
otherwise `checkout` could move or invalidate the very item `found` points at,
leaving `found` dangling. This is exactly the aliasing rule (shared xor
mutable) that makes the whole crate safe without a garbage collector.

### Q&A

1. **Why is `LoanStatus` an enum rather than a `bool` plus two `Option`
   fields?** A `bool` plus two `Option<u32>` fields would be three
   independent boxes that can be filled in inconsistently — nothing stops
   `on_loan = false` while `member_id = Some(100)`, which claims an item is
   both on the shelf and checked out at once. An enum attaches `member_id`
   and `day_borrowed` only to the `OnLoan` variant, so `Available` and `Lost`
   have no fields to fill in incorrectly — the invalid state can't even be
   written down. Matching on an enum is also exhaustive: the compiler forces
   every variant to be handled (or an explicit `_`), so a case like "item is
   lost" can't be silently forgotten the way an untracked boolean could be.

2. **What does `match` force you to do when a fourth `MediaKind` is added
   later?** Every non-wildcard `match` on `MediaKind` — `loan_days`,
   `daily_late_fee_cents`, and the `Display` impl — stops compiling until the
   new variant is added to it. The compiler points at each site individually,
   so there's no way to add a new media kind and forget to decide its loan
   length, its late fee, or how it prints.

3. **`Item::new` takes `String` rather than `&str`. Who owns the title
   afterwards?** The `Item` does. Passing a `String` moves it into the
   constructor, which stores it in the `title` field; the caller no longer
   has access to that value. Taking `&str` instead would require the `Item`
   to either borrow (needing a lifetime tied to the caller's string, which
   the library couldn't outlive) or clone on every construction.

4. **Why does `add_item` take `self` by `&mut` but `item` by value?**
   `&mut self` because pushing onto `self.items` mutates the `Library`'s own
   `Vec`. `item` by value because the `Library` needs to become the
   long-term owner of the `Item` (and its heap-allocated `title`/`author`
   strings) — an item that outlives the call that created it can't be
   borrowed from the caller's stack frame.

5. **When `add_item` returns `Err`, what happened to the `Item` the caller
   passed in?** It was moved into `add_item` and, since the function returns
   before pushing it anywhere, it's dropped at the end of the function —
   its `String` fields are deallocated and the data is gone. The caller has
   no way to get it back. That's an acceptable trade for this assignment
   (duplicate ids and empty titles are checked before any real work, so
   there's nothing to lose besides the two strings), but a friendlier API
   for expensive-to-construct items could return the item back on failure,
   e.g. `Result<(), (LibraryError, Item)>`, so a caller can fix one field and
   retry without rebuilding everything.

6. **Why does `find_item` return `Option<&Item>` rather than
   `Option<Item>`?** The `Library` is the sole owner of every `Item`; it has
   to keep living in `self.items` for other lookups and for `checkout`/
   `return_item` to mutate later. Returning `&Item` lets a caller read the
   item without cloning it or removing it from the vector, and the borrow
   checker guarantees that borrow can't outlive the library or coexist with
   a conflicting mutation.

7. **What is the lifetime `'a` in `items_by_author` actually saying?** It
   ties the lifetime of every reference in the returned `Vec<&'a Item>` to
   the lifetime of the `&'a self` borrow used to produce them. In other
   words: the items handed back are only valid for as long as this
   particular shared borrow of the library is alive — the caller can't hold
   onto them past that borrow ending (e.g. past a later `&mut self` call).

8. **Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the
   same `Library` at once, and how did you structure the method around
   that?** Methods like `find_item`/`find_member` take `&self`, borrowing
   the whole `Library`, not just one field — so two such calls that each
   wanted a `&mut Library` couldn't have overlapping mutable borrows; the
   second call wouldn't compile while the first mutable borrow was still
   live. `checkout` sidesteps this by validating everything through cheap
   immutable lookups first (`find_item`, `find_member`), letting those
   borrows end, and only then reaching in a second time with
   `self.items.iter_mut()` and `self.members.iter_mut()` directly on two
   distinct fields of `self` — which the borrow checker allows
   simultaneously because they're disjoint fields accessed directly, not
   through two overlapping whole-`self` method calls.

9. **Why are `Library`'s fields private?** So the only code that can change
   an item's `LoanStatus` or a member's `borrowed_item_ids` is `Library`'s
   own methods. `checkout` and `return_item` update both together in one
   place; if the fields were public, outside code could set an item to
   `OnLoan` without adding it to a member's list (or vice versa), and the
   two would drift out of sync.

10. **What duplication does the provided `late_fee_cents` remove, and what
    would you lose by making it a free function instead?** Both `MediaKind`
    and `Item` need the same formula —
    `days_held.saturating_sub(loan_days) * daily_late_fee_cents` — so putting
    it once as a default method on the `LoanTerms` trait means neither impl
    repeats it, and any future type implementing `LoanTerms` gets it for
    free. A free function computing the same thing would still work, but
    callers would lose the ability to call `item.late_fee_cents(days)`
    through the trait — they'd have to pull `loan_days()` and
    `daily_late_fee_cents()` out themselves and pass them in, and a type
    couldn't override the formula for itself if it ever needed a different
    late-fee rule.

11. **Why is `Result` preferable to `panic!` for validation failures? Name a
    place in this crate where a panic would be defensible.** Bad input from
    a caller — an unknown id, a duplicate registration, checking out an
    already-loaned item — is an expected, recoverable outcome, not a bug.
    `panic!` would unwind or abort the whole program over one bad request;
    `Result` forces the caller (via the type system) to handle the failure
    and lets everything else keep working. A defensible panic in this crate
    is the `.unwrap()` on `self.items.iter_mut().find(|i| i.id ==
    item_id)` inside `checkout`/`return_item`: by that point `find_item`
    has already confirmed the id exists, so failing to find it again would
    mean the `Library`'s own two collections have fallen out of sync — an
    internal bug, not bad caller input, and exactly the kind of "should be
    impossible" invariant a panic is for.

12. **Which derive did you deliberately leave off a type, and why?** Neither
    `Item` nor `Member` derives `Clone`. The `Library` is meant to be the
    only owner of each one — callers only ever get `&Item`/`&Member` back
    from the lookup methods. If `Clone` were available, it would be tempting
    to clone an item, mutate the clone, and end up with a copy that has
    silently drifted from the library's real state (the exact
    status/borrowed-list desync the private fields in question 9 are meant
    to prevent).

## Design notes

`checkout` and `return_item` are the only two places that ever write to an
item's `status` or a member's `borrowed_item_ids`, and each does so in one
function body: `checkout` sets `LoanStatus::OnLoan` and pushes the item id
onto the member's list in the same call; `return_item` sets the status back
to `Available` and removes the id from the list in the same call. Both
methods validate everything first (ids exist, item isn't lost, item isn't
already on loan / is on loan, borrow limit, checked-subtraction for the
return day) and only mutate after every check has passed, so a rejected
`checkout` or `return_item` never leaves the item and the member half-updated.

`filter_items` (Part 9) is implemented as `fn filter_items<F: Fn(&Item) ->
bool>(&self, predicate: F) -> Vec<&Item>`, and both `items_by_author` and
`available_items` are now one-line calls into it with a closure — the author
comparison and the `status == Available` comparison are the only things that
differ between them.

## Example output

```
Dune was returned on time; fee owed: 0 cents
Project Hail Mary was returned 6 days late; fee owed: 150 cents
expected error checking out item 999: no item with id 999 was found
#1: "Dune" by Frank Herbert (Book (320 pages)) — Available
#2: "Project Hail Mary" by Andy Weir (Audiobook (540 minutes)) — Available
#3: "The Rust Programming Language" by Steve Klabnik (Ebook (1200 kb)) — Available
```