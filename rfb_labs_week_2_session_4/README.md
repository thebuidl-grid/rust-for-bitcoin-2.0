# Rust for Bitcoin 2.0 — Week 2, Session 4

Build a small lending library while practising structs, enums, traits, ownership, borrowing, collections, and `Result`-based error handling. No Bitcoin and no external crates — just Rust.

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

---

## Written answers

### Part 7 Ownership Experiments

#### Experiment A: Reading `item.title` after `library.add_item(item)?`

When trying to read `item.title` after passing `item` to `library.add_item(item)?`:

```text
error[E0382]: borrow of moved value: `item`
  --> src/main.rs:24:30
   |
13 |     let item = Item::new(1, "Dune".into(), "Frank Herbert".into(), MediaKind::Book { pages: 412 });
   |         ---- move occurs because `item` has type `Item`, which does not implement the `Copy` trait
23 |     library.add_item(item)?;
   |                      ---- value moved here
24 |     println!("Title: {}", item.title);
   |                           ^^^^^^^^^^ value borrowed here after move
```

**Why it happened:**  
`add_item` takes `item` by value, moving ownership of the struct into the library. `Item` doesn't implement `Copy`, so `item` is no longer valid in `main` after being moved.

---

#### Experiment B: Holding `library.find_item(1)` across `library.checkout(...)`

When holding `let held = library.find_item(1);` while calling `library.checkout(1, 100, 5)?;`:

```text
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
  --> src/main.rs:37:5
   |
36 |     let held = library.find_item(1);
   |                ------- immutable borrow occurs here
37 |     library.checkout(1, 100, 5)?;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
38 |     println!("Held: {:?}", held);
   |                            ---- immutable borrow later used here
```

**Why it happened:**  
`find_item` borrows `library` immutably to return `&Item`. `checkout` needs a mutable reference `&mut library` to update the items and members lists. Rust doesn't allow a mutable reference while an immutable reference is still active in the same scope.

---

### Questions & Answers

#### 1. Why is `LoanStatus` an enum rather than a `bool` plus two `Option` fields?
Using an enum (`Available`, `OnLoan { member_id, day_borrowed }`, `Lost`) prevents invalid data combinations. With a `bool` and two `Option` fields, you could have `is_borrowed: false` but still store a borrower ID. The enum makes sure borrower data exists only when an item is actually on loan.

#### 2. What does `match` force you to do when a fourth `MediaKind` is added later?
Because `match` in Rust must cover all possibilities, adding a new `MediaKind` variant causes the compiler to flag every unhandled `match` block across the code. This ensures you update loan lengths, fees, and display formatting for the new media type before the code will compile.

#### 3. `Item::new` takes `String` rather than `&str`. Who owns the title afterwards?
The created `Item` struct takes ownership of the `String` memory.

#### 4. Why does `add_item` take `self` by `&mut` but `item` by value?
It takes `&mut self` because it mutates the library's internal items vector. It takes `item` by value so the library becomes the new owner of that item.

#### 5. When `add_item` returns `Err`, what happened to the `Item` the caller passed in? Was that a good design choice, and what is the alternative?
The `Item` was moved into `add_item`, so when the function returns an `Err`, that `Item` goes out of scope and gets dropped. An alternative design would be taking `item` by value and returning `Result<(), (Item, LibraryError)>` so the caller gets the item back on failure, or validating inputs before taking ownership.

#### 6. Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?
It returns a borrowed reference `Option<&Item>` so the library keeps ownership of its items. Returning `Option<Item>` by value would either remove the item from the vector or require cloning it.

#### 7. What is the lifetime `'a` in `items_by_author` actually saying?
It specifies that the returned item references (`&'a Item`) are valid for as long as the borrowed `Library` reference (`&'a self`) lives.

#### 8. Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the same `Library` at once, and how did you structure the method around that?
You can't take two separate `&mut` references into the same struct at the same time. To avoid this, `checkout` checks all conditions first using immutable references (`find`), and then updates the item and member using separate `iter_mut()` calls once validation passes.

#### 9. Why are `Library`'s fields private?
Making `items` and `members` private prevents code outside `Library` from modifying them directly. This keeps item loan statuses and member borrowed lists in sync.

#### 10. What duplication does the provided `late_fee_cents` remove, and what would you lose by making it a free function instead?
`late_fee_cents` provides a default implementation for calculating late fees using `loan_days()` and `daily_late_fee_cents()`. If it were a standalone function, you would lose trait implementation defaults and method syntax (`item.late_fee_cents(days)`).

#### 11. Why is `Result` preferable to `panic!` for validation failures? Name a place in this crate where a panic would be defensible.
Validation errors like missing items or borrow limits are normal operational failures that caller code should handle. Panicking crashes the process. A panic (or `.unwrap()`) is acceptable in `checkout` or `return_item` during the mutation step right after confirming the item or member exists during validation.

#### 12. Which derive did you deliberately leave off a type, and why?
`Clone` and `Copy` were left off `Item` and `Member` because items and members represent real-world objects with unique IDs. Deriving `Clone` could lead to duplicate items or members in memory.

---

## Design notes

- Implemented two-phase handling in `checkout` and `return_item`: validate state first with immutable lookups, then perform mutations.
- Added generic `filter_items` method to reusable item filtering:
  ```rust
  pub fn filter_items<F>(&self, predicate: F) -> Vec<&Item>
  where
      F: Fn(&Item) -> bool,
  {
      self.items.iter().filter(|i| predicate(i)).collect()
  }
  ```

---

## Example output

```text
=== Community Lending Library Stocked ===
Found item: #1 "Dune" by Frank Herbert [Book (412 pages)] - Available

--- Checking out item #1 (Dune) to Ada Lovelace (Member #100) on day 5 ---
Updated status: #1 "Dune" by Frank Herbert [Book (412 pages)] - On loan to member #100 since day #5

--- Returning item #1 on day 35 (Late Return) ---
Item returned successfully. Late fee charged: 225 cents ($2.25)
Final status: #1 "Dune" by Frank Herbert [Book (412 pages)] - Available

--- Attempting invalid checkout (Item #999 does not exist) ---
Handled error Display output: "Item with id 999 not found"
```
