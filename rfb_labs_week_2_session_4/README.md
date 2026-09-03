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

1. Why is `LoanStatus` an enum rather than a `bool` plus two `Option` fields?

LoanStatus` is an enum because it provides a variable and clear description of what the LoanStatus represents. I did a little research and discovered what is termed as the "illegal states unrepresentable rule" which an enum provides. Just like being both available and onloan at the same time It saves the stress where all states are captured during compile time rather than at runtime. Using an enum makes illegal states unrepresentable at compile time. 
   
   If we had a struct like structure for example 
   struct LoanStatus{
    is_available: bool,
    member_id:Option<u32>,
    day_borrowed
   }
    we could represent invalid combinations like:
   - `is_available = true`  member_id = Some(10)` but 'day_borrowed:None'
   - `is_available = false` but member_id = None and day_borrowed: Some(12)
   
   The enum enforces that when an item is `OnLoan`, it MUST have both a member_id and day_borrowed, and when it's `Available` or `Lost`, it cannot have that data.

2. What does `match` force you to do when a fourth `MediaKind` is added later?

When pattern matching on an enum with `match`, it enforces exhaustive coverage of all variants. If someone adds a fourth variant like `MediaKind::Magazine { issues: u32 }` later, every existing `match` statement on `MediaKind` throughout the codebase will fail to compile with an error about non-exhaustive patterns.This forces you to explicitly handle the new variant in every location where MediaKind is matched.

3. `Item::new` takes `String` rather than `&str`. Who owns the title afterwards?

   The `Item` owns the title. Taking `String` by value transfers ownership from the caller to the `Item`, which stores it in its `title` field.

4. Why does `add_item` take `self` by `&mut` but `item` by value?

   `&mut self` allows the library to modify its internal `Vec<Item>` without the caller giving up the library itself. `item` is taken by value because the library needs to own and store it permanently.

5. When `add_item` returns `Err`, what happened to the `Item` the caller passed in? Was that a good design choice, and what is the alternative?

   The item is dropped and lost. This is acceptable for validation errors like empty titles. The alternative is returning `Result<(), (LibraryError, Item)>` to give back the item on error, but that complicates the API for little benefit.

6. Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?

   Returning `&Item` borrows rather than clones or moves. Cloning would be expensive, and moving would remove the item from the library. Borrowing allows inspection while the library retains ownership.

7. What is the lifetime `'a` in `items_by_author` actually saying?

   The returned references live as long as the library borrow. The `'a` ties the lifetime of returned `&Item` references to the lifetime of `&self`, ensuring they can't outlive the library.

8. Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the same `Library` at once, and how did you structure the method around that?

   Rust prevents multiple mutable borrows of the same struct, even if borrowing different fields. I validated everything with immutable borrows first, then mutated item and member separately using index-based access.

9. Why are `Library`'s fields private?

   To maintain invariants. The item's `LoanStatus` and member's `borrowed_item_ids` must stay synchronized. Public fields would let external code break this relationship by modifying one without the other.

10. What duplication does the provided `late_fee_cents` remove, and what would you lose by making it a free function instead?

    It removes duplicating the fee formula in both `MediaKind` and `Item` implementations. As a free function, you'd lose the trait-based polymorphism—you couldn't call the same method on both types.

11. Why is `Result` preferable to `panic!` for validation failures? Name a place in this crate where a panic would be defensible.

    `Result` lets callers handle errors; `panic!` crashes the program. Validation failures like duplicate IDs are expected conditions, not bugs. A defensible panic: after validating an item exists, using `.unwrap()` to get mutable access is safe.

12. Which derive did you deliberately leave off a type, and why?

    `Clone` is deliberately omitted from `Item` and `Member`. The library should be the sole owner of these types. Allowing clones would make it harder to maintain invariants.

## Design notes

**Keeping item status and member list synchronized:**
I used a two-phase approach in both `checkout` and `return_item`:
1. Validate everything with immutable borrows first
2. Only mutate after all checks pass, updating both item status and member list in sequence

This ensures either both updates happen or neither does. The validation order specified in the assignment prevents partial state changes on error.

**Key choices:**
- Validation order matters: checking for item/member existence before checking business rules provides better error messages
- Used `.unwrap()` after validation is safe—we already confirmed items/members exist
- Checked arithmetic prevents integer underflow when validating return dates

## Example output
Paste the output of `cargo run` here once Part 8 is complete.

sarki@bukharee:~/rust-for-bitcoin-2.0/rfb_labs_week_2_session_4$ cargo run
   Compiling rfb_labs_week_2_session_4 v0.1.0 (/home/sarki/rust-for-bitcoin-2.0/rfb_labs_week_2_session_4)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.10s
     Running `target/debug/rfb_labs_week_2_session_4`
  status: On loan to member 100 since day 10
  Late fee charged: 225 cents ($2.25)
  New status: On loan to member 100 since day 10

  Fee: 0 cents (ebooks never charge late fees!)
error: Item with ID 999 not found

error: The requested item with ID 2; already on loan to member with ID 100
Error: ItemAlreadyOnLoan { id: 3, member_id: 101 }




## Part 7: Ownership Experiments

### Experiment A: Using item after moving it to library

**Code attempted:**
```rust
let item = Item::new(
    1,
    "Atomic Habits".to_string(),
    "James Clear".to_string(),
    MediaKind::Book { pages: 320 }
);
library.add_item(item)?;
println!("{}", item.title); // This line causes the error
```

**Compiler error:**
```
error[E0382]: borrow of moved value: `item`
  --> src/main.rs:12:20
   |
10 |     let item = Item::new(1, "Atomic Habits".to_string(), "James Clare".to_string(), MediaKind::Book{pag...
   |         ---- move occurs because `item` has type `Item`, which does not implement the `Copy` trait
11 |     library.add_item(item)?;
   |                      ---- value moved here
12 |     println!("{}", item.title);
   |                    ^^^^^^^^^^ value borrowed here after move

For more information about this error, try `rustc --explain E0382`.
error: could not compile `rfb_labs_week_2_session_4` (bin "rfb_labs_week_2_session_4") due to 1 previous error
```

**Explanation:**
The `add_item` method takes ownership of the `Item` by accepting it by value (`item: Item`). When we call `library.add_item(item)`, ownership of the item is transferred (moved) from the caller into the library's internal `Vec<Item>` storage. After the move, the original `item` variable is no longer valid and cannot be accessed. Attempting to read `item.title` fails because we no longer own that data—the library does. This is Rust's ownership system preventing use-after-move bugs at compile time. The item now lives inside the library and can only be accessed through the library's methods like `find_item()`.

---

### Experiment B: Holding immutable reference while mutating

**Code attempted:**
```rust
let found = library.find_item(1);
library.checkout(1, 100, 5)?; // This line causes the error
println!("{:?}", found);
```

**Compiler error:**
```
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
  --> src/main.rs:13:5
   |
12 |     let found = library.find_item(1);
   |                 ------- immutable borrow occurs here
13 |     library.checkout(1, 100, 5)?;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
14 |     println!("{:?}", found);
   |                      ----- immutable borrow later used here
```

**Explanation:**
The `find_item` method returns `Option<&Item>`, which is an immutable reference into the library's data. The `found` variable holds this reference, keeping an immutable borrow of `library` active throughout its lifetime. When we then try to call `checkout`, which requires a mutable borrow of `library` (`&mut self`), the compiler rejects this because Rust's borrowing rules prohibit having a mutable borrow while an immutable borrow exists. 

This prevents data races and reference invalidation: if `checkout` were allowed to modify the library while `found` points into it, `found` could become a dangling reference or point to inconsistent data (the item's status might change from Available to OnLoan, but `found` would still show the old status). 

