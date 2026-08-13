# Rust for Bitcoin 2.0 — Week 2, Session 4

Build a small lending library while practising structs, enums, traits,
ownership, borrowing, collections, and `Result`-based error handling. No
Bitcoin and no external crates — just Rust.

## Recommended workflow

```bash
cargo test
cargo run
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

## Written answers

**1. Why is `LoanStatus` an enum rather than a `bool` plus two `Option` fields?**

A `bool` plus `Option<u32>` plus `Option<u32>` creates impossible states: the
bool could be false while the options are `Some`, or true while they are `None`.
The compiler cannot rule those combinations out, so every reader must either
ignore them or add defensive checks. An enum makes the invalid states
unrepresentable. `Available`, `OnLoan { member_id, day_borrowed }`, and `Lost`
are the only states that can ever exist, and you always get exactly the data
that belongs to each state and nothing more.

**2. What does `match` force you to do when a fourth `MediaKind` is added later?**

`match` is exhaustive by default. Adding a new variant to `MediaKind` will cause
a compile error in every `match` that does not handle it. That forces you to
visit every decision point in the codebase and consciously decide what the new
variant should do there. Nothing can silently fall through to a wrong default.

**3. `Item::new` takes `String` rather than `&str`. Who owns the title afterwards?**

`Item` owns it. The `String` is moved into the struct field. The caller gives up
ownership at the call site, so no separate lifetime needs to be tracked and the
item can be stored in a collection and outlive the scope that created it.

**4. Why does `add_item` take `self` by `&mut` but `item` by value?**

`&mut self` is the minimum required to mutate the library's internal `Vec`.
`item` is taken by value (moved in) because the library needs to own the item
permanently. Taking it by value is the clearest contract: after the call the
caller no longer has the item; the library does.

**5. When `add_item` returns `Err`, what happened to the `Item` the caller passed in?**

It was dropped at the end of the function body. The item is moved in at the
call site; if validation fails the function returns the error and the item falls
out of scope, so its memory is freed immediately. The caller cannot recover it.

Whether that is a good design depends on context. For this exercise it is fine:
a rejected item is typically a programming error (empty title or duplicate id)
and the caller can simply construct a corrected one. The alternative is to
return the item back to the caller inside the `Err`, e.g.
`Err((LibraryError::EmptyTitle, item))`, which lets the caller reuse the
allocation but complicates the error type.

**6. Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?**

The library owns the item and needs to keep it. Returning `Option<Item>` would
move the item out of the `Vec`, removing it from the collection. A reference
lets the caller read the item while the library retains ownership.

**7. What is the lifetime `'a` in `items_by_author` actually saying?**

It ties the lifetimes of the returned references to the lifetime of the library
borrow. The annotation `&'a self` and `Vec<&'a Item>` says: the references in
the returned vector are valid for exactly as long as `self` is borrowed. The
compiler uses this to ensure the vector cannot outlive the library, preventing
dangling pointers.

**8. Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the same `Library` at once?**

Rust's borrow checker forbids two mutable borrows into the same owner at the
same time, even if they point to different elements. Both items and members live
inside `Library`, so the compiler conservatively treats a `&mut Item` and a
`&mut Member` as potentially aliasing. The workaround is to separate reads from
writes: first validate using shared references (which can coexist), then release
those borrows, and finally mutate by index so no named reference is held across
the two updates.

**9. Why are `Library`'s fields private?**

To enforce the invariant that an item's `LoanStatus` and its borrower's
`borrowed_item_ids` list always agree. If `items` and `members` were public,
any caller could update one without the other. Making the fields private means
the only paths to mutation are `checkout` and `return_item`, both of which
update both sides atomically.

**10. What duplication does the provided `late_fee_cents` remove?**

Both `MediaKind` and `Item` implement `LoanTerms`. Without the default method,
each impl would need to repeat the formula
`days_held.saturating_sub(loan_days()) * daily_late_fee_cents()`. By writing it
once as a default on the trait, the formula lives in one place. If it changed
(say, a grace period were added) only one line would need updating.

Making it a free function instead would lose the polymorphism: you could no
longer call `item.late_fee_cents(days)` or `kind.late_fee_cents(days)` uniformly.
Callers would have to manually pass `loan_days()` and `daily_late_fee_cents()`
as arguments, spreading the formula's knowledge across call sites.

**11. Why is `Result` preferable to `panic!` for validation failures?**

`panic!` unwinds the thread and crashes the program. Callers have no way to
recover or report the error gracefully. `Result` makes the failure part of the
function's type: the caller is forced by the type system to handle it, can
display a useful message, and can continue running. For expected conditions like
a duplicate id or an unknown member, crashing is never appropriate.

A place where a panic would be defensible is the `unwrap()` calls inside
`checkout` and `return_item` after the index lookups. By the time we call
`.position()` on an id that already passed the `find_item`/`find_member`
validation step, we know the item exists. A `None` there would be a bug in the
library itself, not a caller error, so panicking would correctly signal an
internal invariant violation.

**12. Which derive did you deliberately leave off a type, and why?**

`Clone` was not derived on `Item`. The library is built around ownership:
`add_item` takes an `Item` by value so the library is the sole owner. If `Item`
were `Clone`, callers could silently duplicate an item, ending up with two
copies whose `LoanStatus` fields could diverge. Leaving `Clone` off makes that
mistake a compile error.

## Ownership experiments

### Experiment A: read `item.title` after `library.add_item(item)`

```rust
let item = Item::new(1, "Dune".into(), "Frank Herbert".into(), MediaKind::Book { pages: 320 });
let _ = library.add_item(item);
println!("{}", item.title); // compile error here
```

Compiler error:

```
error[E0382]: borrow of moved value: `item`
   --> src/library.rs:178:20
    |
176 |     let item = crate::catalogue::Item::new(1, "Dune".into(), "Frank Herbert".into(), MediaKind::Book { pages: 320 });
    |         ---- move occurs because `item` has type `Item`, which does not implement the `Copy` trait
177 |     let _ = library.add_item(item);
    |                              ---- value moved here
178 |     println!("{}", item.title); // Experiment A: use after move
    |                    ^^^^^^^^^^ value borrowed here after move
```

`add_item` takes `item` by value, so ownership is transferred to the library at
the call site. The variable `item` no longer owns anything. Trying to read
`item.title` afterwards is a use-after-move: the compiler rejects it at compile
time rather than allowing a dangling access.

### Experiment B: hold `find_item` result across `checkout`

```rust
let found = library.find_item(1); // immutable borrow of library
library.checkout(1, 100, 0).unwrap(); // mutable borrow of library -- conflict
println!("{:?}", found); // immutable borrow used here
```

Compiler error:

```
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
   --> src/library.rs:188:5
    |
187 |     let found = library.find_item(1); // immutable borrow occurs here
    |                 ------- immutable borrow occurs here
188 |     library.checkout(1, 100, 0).unwrap(); // mutable borrow — conflict!
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
189 |     println!("{:?}", found);
    |                      ----- immutable borrow later used here
```

`find_item` returns `&Item`, which is an immutable borrow of the library. That
borrow is still alive at the `println!` on the next line. `checkout` needs a
`&mut self`, which is a mutable borrow of the same library. Rust forbids an
active mutable borrow to coexist with any other borrow. The fix is to not hold
the reference across the mutation: copy the data you need out of the reference
before calling `checkout`, or call `find_item` again after the checkout.

## Design notes

The core challenge in `checkout` and `return_item` is that both the item and
the member must be updated together, but the borrow checker will not allow a
`&mut Item` and a `&mut Member` to be held simultaneously from the same
`Library`. The solution is a two-phase approach: validate using shared (`&`)
references first, release all borrows, then mutate using index-based access
(`self.items[idx]`) where no live named reference exists. This pattern keeps the
invariant that item status and member borrowed-id list always agree, because
both mutations happen in the same function with no possibility of an early
return between them once validation passes.

`filter_items` (Part 9) takes a `Fn(&Item) -> bool` closure and collects
matching references. Both `items_by_author` and `available_items` are expressed
as calls to it, so the iteration and collection logic lives in one place.

## Example output

```
=== Library stocked ===
  [1] "Dune" by Frank Herbert — Book (320 pages) — Available
  [2] "The Rust Programming Language" by Steve Klabnik — Ebook (1200 KB) — Available
  [3] "Project Hail Mary" by Andy Weir — Audiobook (540 minutes) — Available

Ada checked out item 1 on day 0.
Ada returned item 1 on day 20. Fee owed: 0 cents (on time).

Ada checked out item 1 again on day 30.
Ada returned item 1 on day 65. Fee owed: 350 cents (14 days late).

Handled error: item 1 is already on loan to member 100
```
