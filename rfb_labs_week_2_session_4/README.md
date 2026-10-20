# Rust for Bitcoin 2.0 — Week 2, Session 4

Build a small lending library while practising structs, enums, traits,
ownership, borrowing, collections, and `Result`-based error handling. No
Bitcoin and no external crates — just Rust.

## Written answers

1. **Why is `LoanStatus` an enum rather than a `bool` plus two `Option` fields?**

`LoanStatus` is an enum because an item can be in one of several distinct states: available, on loan, or lost. The `OnLoan` variant can also store the member id and borrowing day. This prevents invalid combinations of state.

2. **What does `match` force you to do when a fourth `MediaKind` is added later?**

`match` requires every enum variant to be handled. If another `MediaKind` is added, the compiler will identify matches that are no longer exhaustive, so the new variant must be handled.

3. **`Item::new` takes `String` rather than `&str`. Who owns the title afterwards?**

The `Item` owns the `String` after it is passed to `Item::new`. Ownership of the string is moved into the `Item`.

4. **Why does `add_item` take `self` by `&mut` but `item` by value?**

`&mut self` lets the existing library be modified without moving ownership of the library. `item` is passed by value because the library needs to take ownership of it and store it.

5. **When `add_item` returns `Err`, what happened to the `Item` the caller passed in? Was that a good design choice, and what is the alternative?**

Because `add_item` accepts the item by value, ownership is transferred to the function. If validation fails, the item is dropped when the function returns. This is simple, but an alternative would be to return the rejected item inside the error so the caller could recover it.

6. **Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?**

It returns a borrowed reference so the library keeps ownership of the item. This avoids moving or cloning the item just to inspect it.

7. **What is the lifetime `'a` in `items_by_author` actually saying?**

The lifetime says that the returned references are valid for the same lifetime as the borrow of the library. The references cannot outlive the library they came from.

8. **Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the same `Library` at once, and how did you structure the method around that?**

Rust prevents overlapping mutable borrows when the compiler cannot prove that they are independent. I avoided this by first finding the item and member indexes, performing all validation, and only then mutating the collections by index.

9. **Why are `Library`'s fields private?**

The fields are private so callers cannot directly modify an item's status or a member's borrowed-item list independently. The library's methods control these changes and keep the two pieces of state consistent.

10. **What duplication does the provided `late_fee_cents` remove, and what would you lose by making it a free function instead?**

`late_fee_cents` provides the shared late-fee calculation for the different types that implement `LoanTerms`, so the formula does not need to be repeated. A free function could calculate the fee, but the common trait abstraction would be less useful.

11. **Why is `Result` preferable to `panic!` for validation failures? Name a place in this crate where a panic would be defensible.**

`Result` allows expected failures such as an unknown item, duplicate id, or borrow limit to be handled by the caller without terminating the program. A panic could be defensible for an internal programmer-only invariant that should never be violated.

12. **Which derive did you deliberately leave off a type, and why?**

I deliberately left `Copy` off `Item`. An `Item` owns `String` values, and its ownership should be moved explicitly rather than implicitly copied.

## Part 7 — Ownership experiments

### Experiment A

The experiment attempted to use `item` after passing it to `add_item`.

```text
error[E0382]: borrow of moved value: `item`
--> src\main.rs:15:40
|
6 |     let item = Item::new(
|         ---- move occurs because `item` has type `Item`, which does not implement the `Copy` trait
...
13 |     library.add_item(item)?;
|                      ---- value moved here
14 |
15 |     println!("Title after adding: {}", item.title);
|                                        ^^^^^^^^^^ value borrowed here after move
```

The error occurs because `add_item` takes ownership of the `Item`. After the item is passed to the library, the variable `item` can no longer be used.

### Experiment B

The experiment held an immutable reference from `find_item`, then tried to mutably borrow the library with `checkout`.

```text
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
--> src\main.rs:17:5
|
15 |     let held = library.find_item(1);
|                ------- immutable borrow occurs here
16 |
17 |     library.checkout(1, 100, 10)?;
|     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
18 |
19 |     println!("Held item: {:?}", held);
|                                 ---- immutable borrow later used here
```

The error occurs because `held` contains a reference into `library`. That immutable borrow is still active when `checkout` tries to mutably borrow the same library.

## Design notes

The library owns all items and members, while its fields remain private. This ensures that changes to an item's `LoanStatus` and a member's borrowed-item list happen through the library methods.

For checkout, I validate the item first, then the member, then the item's status, and finally the member's borrow limit. Only after all validation succeeds do I update the item and member. This prevents partially completed operations.

For returns, I use checked subtraction to calculate the days held, calculate the late fee through `LoanTerms`, set the item back to `Available`, and remove its id from the member's borrowed list.

The lookup methods return borrowed references rather than cloning stored items. I did not implement the optional generic `filter_items` feature because it was not required for the main assignment.

## Example output

```text
Available items:
#1: Dune by Frank Herbert [Book (320 pages)] - Available
#2: Project Hail Mary by Andy Weir [Audiobook (540 minutes)] - Available
#3: Rust in Action by Tim McNamara [Ebook (1500 KB)] - Available
Checked out: #1: Dune by Frank Herbert [Book (320 pages)] - On loan to member 100 since day 10
Returned Dune on day 40; late fee: 225 cents
Handled error: item 1 is not currently on loan
```

## Verification

* `cargo check` — passed
* `cargo test` — 20 passed, 0 failed
* `cargo test -- --ignored — 0 ignored tests to run
* `cargo fmt --check — passed
* `cargo clippy --all-targets --all-features -- -D warnings` — passed
* `cargo run` — passed
