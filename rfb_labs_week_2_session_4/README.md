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

### Part 1: Data Model

1. **Why is `LoanStatus` an enum rather than a `bool` plus two `Option` fields?**

   An enum is more type-safe and expressive. A `bool` plus two `Option<u32>` fields would allow impossible states (e.g., `is_borrowed=true` with both fields `None`, or `is_borrowed=false` with fields populated). The enum enforces at compile time that we can only be in one of three valid states: `Available`, `OnLoan { member_id, day_borrowed }`, or `Lost`. This makes the code safer and makes invalid states unrepresentable.

2. **What does `match` force you to do when a fourth `MediaKind` is added later?**

   Adding a fourth variant to `MediaKind` would cause `match` expressions to fail compilation with a non-exhaustive pattern error. This forces us to handle the new variant everywhere it's used (in `loan_days()`, `daily_late_fee_cents()`, `Display`, etc.), preventing silent bugs where new media types wouldn't be handled properly.

### Part 3-6: Ownership and Implementation

3. **`Item::new` takes `String` rather than `&str`. Who owns the title afterwards?**

   The `Item` owns the title afterwards. Taking `String` by value transfers ownership into the `Item` struct, ensuring the item can exist independently without relying on an external lifetime.

4. **Why does `add_item` take `self` by `&mut` but `item` by value?**

   `self` is taken by `&mut` because we need to mutate the library (push the item into its internal vector). `item` is taken by value because the library takes ownership of it — the caller should not retain access after calling `add_item`, which aligns with the library's responsibility to manage all items.

5. **When `add_item` returns `Err`, what happened to the `Item` the caller passed in? Was that a good design choice, and what is the alternative?**

   When `add_item` returns `Err`, the item is dropped (consumed but not stored). This is a good design choice because invalid items shouldn't be retained—if you try to add an item with an empty title, it makes sense that it's gone. The alternative would be to return the item back to the caller (e.g., `Result<(), (LibraryError, Item)>`), but this complicates error handling.

6. **Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?**

   Returning `&Item` borrows the item from the library, avoiding unnecessary clones. The item is owned by the library and should stay there; callers just need to read it. This is more efficient and prevents duplicated data.

7. **What is the lifetime `'a` in `items_by_author` actually saying?**

   The lifetime `'a` says: "The references in the returned vector are valid as long as `self` is valid." Because the items are owned by the library (borrowed as `&'a self`), any references we return must not outlive that borrow.

8. **Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the same `Library` at once, and how did you structure the method around that?**

   Rust forbids holding two mutable references to data from the same struct because they could alias and violate memory safety. We structured `checkout` to avoid this by finding, validating, and then mutating separately: first we validate with immutable borrows using `find_item` and `find_member`, then after validation passes, we use `iter_mut` to find and mutate the item and member.

### Part 9: Library Design

9. **Why are `Library`'s fields private?**

   Private fields enforce that the library maintains invariants. The library must keep an item's `LoanStatus` and a member's `borrowed_item_ids` list in sync. If fields were public, external code could violate this invariant by modifying them independently, leading to inconsistent state.

10. **What duplication does the provided `late_fee_cents` remove, and what would you lose by making it a free function instead?**

    The default implementation of `late_fee_cents` removes the need for both `MediaKind` and `Item` to repeat the formula `(days_held - loan_days()) * daily_late_fee_cents()`. Making it a free function would lose the ability to call other trait methods (`loan_days`, `daily_late_fee_cents`) implicitly on `self`, forcing explicit parameter passing and making the code less elegant.

11. **Why is `Result` preferable to `panic!` for validation failures? Name a place in this crate where a panic would be defensible.**

    `Result` lets callers decide how to handle errors; `panic!` crashes the program. We use `Result` for expected failures (unknown item, duplicate id, borrow limit). A panic would be defensible only for internal logic errors, such as if we found that an item's `LoanStatus` claimed it was on loan to member X, but member X's list didn't include that item—this indicates a bug in our invariant-maintenance code.

12. **Which derive did you deliberately leave off a type, and why?**

    We left off `Clone` from `Item` because items are owned by the library and should not be duplicated. Leaving it off prevents accidental cloning and enforces that callers borrow items rather than copying them.

---

## Part 7: Ownership Experiments

**Experiment A: Read item.title after library.add_item(item)?**

```
error[E0382]: borrow of moved value: `item`
 --> src/experiments.rs:16:32
  |
12 |         let item = Item::new(...);
   |             ---- move occurs because `item` has type `Item`, which does not implement `Copy`
13 |         library.add_item(item).ok();
   |                          ---- value moved here
14 |         println!("{}", item.title);
   |                        ^^^^ value borrowed after move
```

**Explanation:** `add_item` takes ownership of `item`, moving it into the library. Once moved, the original binding `item` is no longer valid. This prevents use-after-move bugs and ensures the library is the sole owner.

---

**Experiment B: Hold result of find_item(1), call checkout(..), then print**

```
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
 --> src/experiments.rs:34:9
  |
29 |         let held = library.find_item(1);
   |                    ^^^^^^^ immutable borrow occurs here
30 |         library.checkout(1, 100, 0).ok();
   |         ^^^^^^^ mutable borrow occurs here
31 |         println!("{:?}", held);
   |                   ---- immutable borrow later used here
```

**Explanation:** `find_item` returns a reference `&Item` that borrows from `library`. While `held` is in scope, the borrow is active. `checkout` requires a `&mut Library`, but we can't take a mutable borrow while an immutable borrow exists. The compiler prevents potential undefined behavior: if `checkout` reallocated the item vector, `held` would be a dangling pointer.

## Design notes

### Maintaining Invariants

The critical invariant is: **if an item's `LoanStatus` is `OnLoan { member_id, day_borrowed }`, then that member's `borrowed_item_ids` must contain the item's id, and vice versa.**

To maintain this, `checkout` and `return_item` always update both the item and the member in the same method call:

- **`checkout`**: After validating in the correct order, we mutate the item's status and immediately push its id to the member's list.
- **`return_item`**: We mutate the item back to `Available` and simultaneously remove it from the member's list using `retain`.

By keeping mutations together and making the fields private, we prevent external code from breaking the invariant.

### Validation Order in `checkout`

The assignment specifies the error check order: unknown item, unknown member, lost item, already on loan, then borrow limit. This order matters for user experience—a caller fixing one error at a time gets predictable feedback rather than random error messages.

### Optional Part 9: Generic Search

A `filter_items` function was not implemented, but the architecture would support it easily:

```rust
pub fn filter_items<F>(&self, predicate: F) -> Vec<&Item>
where
    F: Fn(&Item) -> bool,
{
    self.items.iter().filter(|item| predicate(item)).collect()
}
```

Then `items_by_author` and `available_items` could be expressed as:
```rust
pub fn items_by_author(&self, author: &str) -> Vec<&Item> {
    self.filter_items(|item| item.author == author)
}

pub fn available_items(&self) -> Vec<&Item> {
    self.filter_items(|item| item.status == LoanStatus::Available)
}
```

---

## Example output

```
Alice checks out 1984...
✓ Item checked out successfully

Alice returns 1984 late (after 30 days)...
✓ Item returned. Late fee: 225 cents ($2.25)

Attempting to register Alice again...
✗ Error: member 101 already registered
```
