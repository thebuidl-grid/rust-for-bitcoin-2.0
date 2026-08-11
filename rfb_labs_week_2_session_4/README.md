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

### 1. Why is `LoanStatus` an enum rather than a `bool` plus two `Option` fields?
Using a `bool` (e.g. `is_loaned`) alongside two `Option` fields (e.g. `borrower_id: Option<u32>` and `day_borrowed: Option<u32>`) introduces illegal, inconsistent states representable in memory—such as `is_loaned = false` with `borrower_id = Some(100)`, or `is_loaned = true` with `day_borrowed = None`. Furthermore, representing additional states like `Lost` would require more booleans and create further uncoordinated combinations. An enum makes illegal states unrepresentable: an item is either `Available`, `OnLoan { member_id, day_borrowed }` (where the loan metadata is guaranteed to exist together), or `Lost`.

### 2. What does `match` force you to do when a fourth `MediaKind` is added later?
Rust's `match` expressions are exhaustive. When a fourth variant is added to `MediaKind` (for example `Magazine { issue: u32 }`), the compiler flags every `match` statement on `MediaKind` (in `loan_days`, `daily_late_fee_cents`, `fmt::Display`, etc.) as incomplete until the new variant is explicitly handled. This prevents unhandled runtime branch bugs at compile time.

### 3. `Item::new` takes `String` rather than `&str`. Who owns the title afterwards?
`Item::new` takes ownership of the `String` and moves it directly into the `title` field of the constructed `Item`. After construction, the `Item` instance is the sole owner of the `title` string and its heap buffer.

### 4. Why does `add_item` take `self` by `&mut` but `item` by value?
`add_item` takes `&mut self` because it modifies the library's internal state (`self.items.push(item)`) without destroying or consuming the `Library` itself. It takes `item` by value (ownership) because the library needs to store and own the `Item` in its collection. Taking `item` by value allows the caller to transfer ownership cleanly without allocating a copy or clone.

### 5. When `add_item` returns `Err`, what happened to the `Item` the caller passed in? Was that a good design choice, and what is the alternative?
Because `add_item` takes `item` by value, when validation fails and an `Err` is returned, the `item` is dropped and deallocated when `add_item`'s scope ends. The caller permanently loses the item they constructed. While simple for the library, it can be inconvenient for callers who might want to inspect, rectify (e.g., set a valid title), or retry. A common Rust alternative is returning `Result<(), (Item, LibraryError)>` so ownership of the rejected item is returned to the caller on failure.

### 6. Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?
The `Library` owns all items in its collection. If `find_item` returned `Option<Item>` by value, it would either have to remove the item from the library or perform a deep clone (allocating new strings for `title` and `author`). Returning `Option<&Item>` grants the caller read access by borrowing directly from `self` with zero allocations and zero copies, leaving the library's contents intact.

### 7. What is the lifetime `'a` in `items_by_author` actually saying?
The signature `fn items_by_author<'a>(&'a self, author: &str) -> Vec<&'a Item>` explicitly connects the lifetime of the returned item references (`&'a Item`) to the lifetime of the borrow of `&'a self`. It informs the compiler that every `&Item` in the returned `Vec` is valid only as long as the `Library` instance remains alive and borrowed. The `author` lifetime is independent and elided because no reference to `author` is retained.

### 8. Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the same `Library` at once, and how did you structure the method around that?
Under Rust's aliasing rules, taking multiple simultaneous mutable borrows (`&mut self.items` and `&mut self.members` or two mutable references into collections on `self`) would violate the rule that only one mutable reference to data in `self` can exist at a time, and risks collection iterator invalidation. We structured `checkout` with a **validate-first, mutate-second** architecture: we first find the indices `item_idx` and `member_idx`, validate all conditions immutably, and then perform sequential index-based mutations (`self.items[item_idx]` and `self.members[member_idx]`).

### 9. Why are `Library`'s fields private?
`Library` must enforce critical domain invariants: an item marked `LoanStatus::OnLoan { member_id, .. }` must stay perfectly synchronized with that member's `borrowed_item_ids` list. If fields were public, outside code could modify items or member lists independently, leading to state drift, bypass of the borrow limit, or duplicate IDs. Private fields ensure all modifications go through validated methods.

### 10. What duplication does the provided `late_fee_cents` remove, and what would you lose by making it a free function instead?
The default implementation of `late_fee_cents` in the `LoanTerms` trait computes `days_held.saturating_sub(self.loan_days()) * self.daily_late_fee_cents()` in a single place so neither `MediaKind` nor `Item` needs to re-implement the formula. If it were a free function, individual implementers would lose the ability to provide custom overrides if needed, and callers would lose the ergonomic method syntax (`item.late_fee_cents(days)`).

### 11. Why is `Result` preferable to `panic!` for validation failures? Name a place in this crate where a panic would be defensible.
`Result` represents expected, recoverable failure conditions (e.g. unknown ID, item already on loan, duplicate ID) and forces callers to explicitly handle them without terminating the thread or process. A `panic!` is defensible only for unrecoverable programmer errors or internal invariant violations (for example, if an internal lookup inside an assertion in a test fails, or if an internal index calculation becomes corrupted).

### 12. Which derive did you deliberately leave off a type, and why?
`Clone` and `Copy` were deliberately omitted from `Library` and `Item` / `Member` (only small scalar enums derive `Copy`). Omitting `Clone` from `Library` prevents accidental expensive deep copies of the entire collection and preserves the semantic idea of a unique physical library. Omitting `Copy` from `Item` and `Member` ensures move semantics and unique ownership of library assets and member records.

---

### Part 7 — Ownership Experiments

#### Experiment A: Reading `item.title` after `library.add_item(item)?`

**Compiler Error:**
```text
error[E0382]: borrow of moved value: `item`
 --> src/main.rs:7:20
  |
5 |     let item = Item::new(1, "Dune".into(), "Frank Herbert".into(), MediaKind::Book { pages: 320 });
  |         ---- move occurs because `item` has type `Item`, which does not implement the `Copy` trait
6 |     library.add_item(item)?;
  |                      ---- value moved here
7 |     println!("{}", item.title);
  |                    ^^^^^^^^^^ value borrowed here after move

For more information about this error, try `rustc --explain E0382`.
```

**Explanation:**
`library.add_item(item)` takes `item` by value, transferring ownership of the `Item` into `library`. Because `Item` does not implement `Copy`, the variable `item` is invalidated after the move, and attempting to read `item.title` results in a use-after-move compiler error (`E0382`).

#### Experiment B: Holding `library.find_item(1)` across `library.checkout(...)`

**Compiler Error:**
```text
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
  --> src/main.rs:9:5
   |
 8 |     let held_item = library.find_item(1);
   |                     ------- immutable borrow occurs here
 9 |     library.checkout(1, 100, 5)?;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
10 |     println!("{:?}", held_item);
   |                      --------- immutable borrow later used here

For more information about this error, try `rustc --explain E0502`.
```

**Explanation:**
`library.find_item(1)` borrows `library` immutably (`&library`) and returns a reference `&Item`. While `held_item` is still alive and used on line 10, calling `library.checkout(...)` requires an exclusive mutable borrow (`&mut library`). Rust's borrow checker enforces that no mutable borrow can coexist with active immutable borrows (`E0502`), preventing memory unsafety and pointer invalidation.

---

## Design notes

1. **State Synchronization & Validation Invariants**:
   - `checkout` and `return_item` follow a strict **validate-first, mutate-second** order.
   - For `checkout`, error checks occur in the exact specified priority: `ItemNotFound` -> `MemberNotFound` -> `ItemIsLost` -> `ItemAlreadyOnLoan` -> `BorrowLimitReached`.
   - Only after all checks pass are `item.status` and `member.borrowed_item_ids` updated in unison, guaranteeing they never drift apart.
   - For `return_item`, checked subtraction (`day.checked_sub(day_borrowed)`) safely detects invalid return days without underflow panics.
2. **Generic Search (Part 9)**:
   - Implemented `Library::filter_items<F>(&self, predicate: F) -> Vec<&Item> where F: Fn(&Item) -> bool`.
   - Both `items_by_author` and `available_items` are cleanly expressed as zero-cost filter closures over this generic method.

---

## Example output

```text
=== Community Lending Library Demo ===

Library catalog stocked successfully:
 - "Dune" by Frank Herbert [Book (320 pages)] - Available
 - "Project Hail Mary" by Andy Weir [Audiobook (540 mins)] - Available
 - "The Rust Programming Language" by Steve Klabnik [Ebook (1200 KB)] - Available

Members registered: Alice (ID: 100), Bob (ID: 101)

Longest allowable loan item: "Dune"

Checking out item 1 (Dune) to member 100 on day 5...
Checkout successful! Item status: "Dune" by Frank Herbert [Book (320 pages)] - On loan to member 100 (borrowed day 5)

Returning item 1 on day 35 (overdue by 9 days)...
Return successful! Late fee owed: 225 cents ($2.25)
Item status after return: "Dune" by Frank Herbert [Book (320 pages)] - Available

Attempting invalid operation: returning an item not on loan (item 2)...
Handled expected error using Display: "item 2 is not currently on loan"

Attempting invalid operation: registering duplicate member ID 100...
Handled expected error using Display: "member with id 100 is already registered"

=== Demo completed successfully ===
```
