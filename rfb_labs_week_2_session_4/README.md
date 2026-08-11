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

1. **Why is `LoanStatus` an enum rather than a `bool` plus two `Option`
   fields?**
   An enum makes the three states — `Available`, `OnLoan { member_id,
   day_borrowed }`, `Lost` — mutually exclusive by construction. With a `bool`
   plus two separate `Option` fields, nothing in the type system would stop
   an invalid combination, like `on_loan = false` while `member_id` is still
   `Some(..)`. The enum makes impossible states genuinely unrepresentable
   instead of just "supposed to not happen."

2. **What does `match` force you to do when a fourth `MediaKind` is added
   later?**
   Every `match` on `MediaKind` (in `loan_days`, `daily_late_fee_cents`, and
   the `Display` impl) would fail to compile until a new arm is added for the
   new variant, unless a wildcard `_` arm is present. This forces every place
   that depends on the full set of media kinds to be revisited, rather than
   silently falling through with the wrong behavior.

3. **`Item::new` takes `String` rather than `&str`. Who owns the title
   afterwards?**
   The `Item` struct owns the `String` after construction. Passing `String`
   moves the caller's string into the new `Item`, so the `Item` doesn't need
   to borrow from — or outlive — anything external. It's fully self-contained.

4. **Why does `add_item` take `self` by `&mut` but `item` by value?**
   `&mut self` because `add_item` needs to mutate the library's internal
   `Vec<Item>` (push into it), but the library itself should keep existing
   afterward. `item` is taken by value because the library needs to *own* the
   item going forward; a borrowed `&Item` would mean the caller still owns it
   and could let it go out of scope, leaving the library holding a dangling
   reference.

5. **When `add_item` returns `Err`, what happened to the `Item` the caller
   passed in? Was that a good design choice, and what is the alternative?**
   Because `item: Item` was already moved into the function by the time
   validation runs, a rejected item is simply dropped — the caller has no way
   to get it back. This is reasonable here (the errors are just "bad id" or
   "empty title," so little is lost), but isn't ideal in general: a caller
   with a large, carefully filled `Item` and one small mistake has to rebuild
   it from scratch. The alternative is returning `Result<(), (Item,
   LibraryError)>` (or an error type that carries the rejected item back), so
   the caller can recover without reconstructing everything.

6. **Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?**
   Returning `Option<Item>` would require either moving the item out of the
   library's `Vec` (leaving a hole, or requiring `Clone`) or cloning it
   outright. Neither is desirable — the library needs to keep owning every
   item, and cloning risks the caller's copy silently drifting out of sync
   with the library's real state. A borrowed `Option<&Item>` lets the caller
   read the data without taking ownership or paying a copy cost.

7. **What is the lifetime `'a` in `items_by_author` actually saying?**
   ```rust
   pub fn items_by_author<'a>(&'a self, author: &str) -> Vec<&'a Item>
   ```
   `'a` says the references returned in the `Vec<&Item>` are valid for exactly
   as long as the borrow of `self` used to create them is valid — the
   returned references can't outlive the library they point into.

8. **Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the same
   `Library` at once, and how did you structure the method around that?**
   Both `Item` and `Member` live inside `self` (`self.items`,
   `self.members`), and Rust's borrow checker only allows one mutable borrow
   of `self`'s data at a time, with no immutable borrows overlapping it.
   `checkout` is structured in two phases: first, every validation check uses
   short-lived immutable lookups (`self.items.iter().find(..)`,
   `self.members.iter().find(..)`) that end as soon as their `if`/`match`
   finishes; only after every check passes does the method perform the
   mutations, using fresh `iter_mut()` lookups done one at a time.

9. **Why are `Library`'s fields private?**
   If `items` and `members` were public, any caller could push, remove, or
   overwrite entries directly — bypassing validation and breaking the
   invariant that an item's `LoanStatus` and a member's `borrowed_item_ids`
   must always agree. Private fields force every change through `Library`'s
   own methods, the only code that can guarantee that invariant holds.

10. **What duplication does the provided `late_fee_cents` remove, and what
    would you lose by making it a free function instead?**
    Without the default trait method, both `impl LoanTerms for MediaKind` and
    `impl LoanTerms for Item` would independently repeat the same "if held
    longer than allowed, charge (days over) × (daily rate)" logic, and any
    future change to that formula would need updating in two places. As a
    free function, it would need `loan_days` and `daily_late_fee_cents` passed
    in separately (or `&dyn LoanTerms`), losing the natural
    `self.late_fee_cents(days_held)` call syntax and no longer visibly
    belonging to the trait's contract.

11. **Why is `Result` preferable to `panic!` for validation failures? Name a
    place in this crate where a panic would be defensible.**
    `Result` lets the caller decide how to respond to an expected, ordinary
    failure — like checking out an item that's already on loan — without
    crashing the program. A panic would only be defensible for a genuine
    logic error that should never happen if the code is correct — for
    example, the `.expect(...)` calls inside `checkout`/`return_item` right
    after mutating, where the item or member's existence was already
    confirmed by an earlier lookup in the same method; if that `.expect` ever
    fired, it would mean an actual bug in the library's own logic, not bad
    input from a caller.

12. **Which derive did you deliberately leave off a type, and why?**
    `Item` does not derive `Clone` or `Copy`. Since `Library` is meant to be
    the sole owner of every `Item`, allowing it to be cloned freely would
    make it easy to accidentally create a second, independent copy whose
    `LoanStatus` could drift out of sync with the library's real record —
    exactly the kind of duplicated, disagreeing state the private-fields
    design in question 9 is meant to prevent.

## Design notes

The core invariant this crate protects is: **an item's `LoanStatus` and its
borrower's `borrowed_item_ids` must always agree.** This is kept true by never
exposing a way to mutate one without the other. `checkout` and `return_item`
are the only two places that change either piece of state, and each of them
updates both together, in the same method call, after all validation has
already passed — there's no intermediate state where one has changed and the
other hasn't. Because `Library`'s fields are private and no other method
offers direct mutable access to `items` or `members`, this invariant can't be
broken from outside the module.

Part 9 (generic `filter_items`) was not attempted.
## Example output

Paste the output of `cargo run` here once Part 8 is complete.
 cargo run
   Compiling rfb_labs_week_2_session_4 v0.1.0 (/home/jemiah/Documents/rust-for-bitcoin-2.0/rust-for-bitcoin-2.0/rfb_labs_week_2_session_4)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s
     Running `target/debug/rfb_labs_week_2_session_4`
--- Catalogue on day 0 ---
#1 "Dune" by Frank Herbert [Book (688 pages)] — available
#2 "Children of Dune" by Frank Herbert [Book (480 pages)] — available
#3 "The Hobbit" by koko. milah [Audiobook (660 min)] — available
#4 "Project Hail Mary" by Abey shittu [Ebook (2048 KB)] — available
#5 "Foundation" by Isaac Asimov [Book (255 pages)] — available

Checked out item 1 to member 100 on day 10.
Returned item 1 on day 15 (on time) — fee owed: 0 cents.

Checked out item 2 to member 100 on day 20.
Returned item 2 on day 45 (late) — fee owed: 100 cents.

--- Items by Frank Herbert ---
#1 "Dune" by Frank Herbert [Book (688 pages)] — available
#2 "Children of Dune" by Frank Herbert [Book (480 pages)] — available

Item with the longest loan window: #5 "Foundation" by Isaac Asimov [Book (255 pages)] — available

Handled error as expected: no item found with id 999

## The two experiments

### Experiment A — read `item.title` after `library.add_item(item)?`

```rust
let item2 = Item::new(
    6,
    "Neuromancer".to_string(),
    "William Gibson".to_string(),
    MediaKind::Book { pages: 271 },
);
library.add_item(item2)?;
println!("{}", item2.title); // <-- should fail to compile
```

```
error[E0382]: borrow of moved value: `item2`
  --> src/main.rs:97:20
   |
90 |     let item2 = Item::new(
   |         ----- move occurs because `item2` has type `Item`, which does not implement the `Copy` trait
...
96 |     library.add_item(item2)?;
   |                      ----- value moved here
97 |     println!("{}", item2.title); // <-- should fail to compile
   |                    ^^^^^^^^^^^ value borrowed here after move
```

`add_item` takes `item: Item` by value, not by reference, so calling
`library.add_item(item2)` transfers ownership of `item2` into the function.
Once moved, `item2` no longer belongs to `main`, and the compiler refuses to
let it be used again — even just to read `.title`. `Item` doesn't implement
`Copy` because it owns heap-allocated `String`s, so Rust can't silently
duplicate it to make the second use free. The move is real and permanent,
which is exactly the guarantee ownership provides: there's no way to
accidentally read data that has already been handed off elsewhere.

### Experiment B — hold `library.find_item(1)`, call `checkout`, then print what was held

```rust
let found = library.find_item(1);
library.checkout(1, 100, 40)?;
println!("{found:?}");
```

```
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
  --> src/main.rs:90:5
   |
90 |     let found = library.find_item(1);
   |                 ------- immutable borrow occurs here
91 |     library.checkout(1, 100, 40)?;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
92 |     println!("{found:?}");
   |                ----- immutable borrow later used here
```

`find_item` returns `Option<&Item>`, a reference borrowed from `library`.
Because `found` is used again later in the `println!`, that immutable borrow
of `library` is still alive at the point `checkout` is called. `checkout`
needs `&mut self`, and Rust never allows a mutable borrow to coexist with an
immutable one on the same value — if it did, `checkout` could invalidate the
very data `found` points to while something still expected to read it safely.
This is the same rule that shaped how `checkout` and `return_item` are written
internally: all reads happen first and are allowed to end, and only then does
mutation happen as a separate step.
