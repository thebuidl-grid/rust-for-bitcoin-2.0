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


### Part 1 — Data model

#### 1. Why is `LoanStatus` an enum rather than a `bool` plus two `Option` fields?

`LoanStatus` is an enum because an item has a distinct state: it can be
`Available`, `OnLoan`, or `Lost`. The `OnLoan` variant can also carry the
member ID and borrowing day. An enum keeps the possible states together and
prevents invalid combinations of fields that could occur with separate
boolean and optional fields.

#### 2. What does `match` force you to do when a fourth `MediaKind` is added later?

`match` requires every possible variant to be handled. If another
`MediaKind` variant is added, existing `match` expressions that do not handle
it will cause a compiler error until the new case is handled.

### Part 2 — Ownership

#### 3. `Item::new` takes `String` rather than `&str`. Who owns the title afterwards?

The `Item` owns the `String` containing its title. When the `String` is passed
into `Item::new`, ownership is transferred to the `Item`.

#### 4. Why does `add_item` take `self` by `&mut` but `item` by value?

`&mut self` allows the library to modify its collection of items. The `item`
is taken by value because the library needs to become the owner of that item
and store it in its `items` collection.

#### 5. When `add_item` returns `Err`, what happened to the `Item` the caller passed in? Was that a good design choice, and what is the alternative?

Because `add_item` takes the item by value, ownership is transferred to the
function when it is called. Even if validation later returns an error, the
caller cannot use the original variable.

This can be reasonable when the library should take ownership regardless of
whether insertion succeeds. An alternative would be to validate using a
borrow first or change the API so that ownership is only transferred after
validation.

#### 6. Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?

It returns a reference because the library already owns the item. Returning
`&Item` lets the caller inspect the item without cloning or transferring
ownership. `Option` represents the possibility that no item with that ID
exists.

#### 7. What is the lifetime `'a` in `items_by_author` actually saying?

The lifetime `'a` says that the references returned by `items_by_author` are
valid for the same lifetime as the borrow of the `Library`. The returned
references cannot outlive the library they point into.

### Part 3 — Borrowing and mutation

#### 8. Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the same `Library` at once, and how did you structure the method around that?

Both the item and member are stored inside the same `Library`. Holding
mutable references into the library while trying to create another mutable
borrow can violate Rust's borrowing rules.

The method first finds the indexes of the item and member and performs all
validation. After validation succeeds, it uses those indexes to mutate the
item and member.

#### 9. Why are `Library`'s fields private?

The library is responsible for keeping an item's loan status and the
member's borrowed-item list consistent. Making the fields private prevents
callers from changing those collections directly and creating inconsistent
state.

### Part 4 — Traits
#### 10. What duplication does the provided `late_fee_cents` remove, and what would you lose by making it a free function instead?

`late_fee_cents` provides the shared formula for calculating how many days an
item is late and multiplying that by its daily fee. Both `MediaKind` and
`Item` can use the same formula instead of implementing it separately.

Making it a free function could still work, but the fee calculation would no
longer be part of the `LoanTerms` interface. The trait groups the loan rules
and the calculation together.

### Part 5 — Error handling

#### 11. Why is `Result` preferable to `panic!` for validation failures? Name a place in this crate where a panic would be defensible.

Validation failures are expected situations that callers may need to handle,
such as an unknown item or a member reaching the borrowing limit. `Result`
allows the caller to handle these failures without terminating the program.

A panic could be defensible for an internal programming invariant that should
never be violated if the code is correct, rather than for normal user input
or library validation failures.

### Part 6 — Derives

#### 12. Which derive did you deliberately leave off a type, and why?

`Copy` was deliberately not derived for `Item`. An `Item` owns a `String`,
and copying ownership of that data automatically would not be appropriate.
The library should own the item and references can be borrowed when the item
needs to be inspected.


## Ownership experiments

### Experiment A

```text
error[E0382]: borrow of moved value: `item`

borrow of moved value: `item`

move occurs because `item` has type `Item`, which does not implement
the `Copy` trait
```

This happened because `library.add_item(item)` takes ownership of the item.
After the item is moved into the library, the original `item` variable cannot
be used.

### Experiment B

```text
error[E0502]: cannot borrow `library` as mutable because it is also
borrowed as immutable

immutable borrow occurs here
mutable borrow occurs here
immutable borrow later used here
```

This happened because `find_item` returns a reference to an item inside the
library. That reference keeps an immutable borrow of the library active.
`checkout` requires a mutable borrow of the same library, so Rust prevents
both borrows from being active at the same time.

## Design notes

Describe any choices you made, including how you kept an item's status and its
borrower's list from drifting apart, and (if attempted) the optional generic
search.

#### notes:
For UTXO selection, I used a simple selection strategy that processes items
in the order they appear. This keeps the implementation straightforward and
easy to reason about.

For the lending library, I kept the `Library` as the owner of both items and
members. The item stores its current `LoanStatus`, while the member stores
the IDs of the items they currently hold. The checkout and return operations
update both pieces of state together so that the two sides remain
consistent.

The implementation uses borrowed references for lookup operations such as
`find_item`, `find_member`, and `items_by_author`, avoiding unnecessary
cloning of the stored data.


## Example output

Paste the output of `cargo run` here once Part 8 is complete.

```text
Returned item 1 late. Fee: 75 cents
Handled error: item with id 999 was not found
```
