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

   > An item's loan state is inherently "one of" — `Available`, `OnLoan`, or `Lost` — never more than one at once. A `bool` plus two `Option` fields can't express that exclusivity in the type itself: nothing stops `is_on_loan: false` from coexisting with `member_id: Some(100)`, an invalid state the compiler would happily accept. The enum makes that combination impossible to construct, so every function that touches `LoanStatus`can trust it's always in exactly one valid shape.

2. What does `match` force you to do when a fourth `MediaKind` is added later?

   > `match` is exhaustive — the compiler rejects the code if any variant is unhandled and there's no `_` catch-all. So adding a fourth `MediaKind` variant (say `Magazine`) doesn't fail quietly at runtime; it fails to *compile*, at every single `match` over `MediaKind` anywhere in the crate that doesn't already have a wildcard arm. The compiler finds every call site that needs a decision for the new variant — there's no way to forget one.

3. `Item::new` takes `String` rather than `&str`. Who owns the title afterwards?

   > `Item::new` takes `title: String` by value, so the `String` is moved into the new `Item` — the `Item` now owns its title outright. If it took `&str` instead, the `Item` would be borrowing someone else's data, which means it could only live as long as whatever it borrowed from — a `library` full of `Item`s couldn't outlive the `String`s the caller built them from. Taking ownership means an `Item` is self-contained and can be stored in the `Library`'s `Vec<Item>` indefinitely, with no lifetime tied to the caller

4. Why does `add_item` take `self` by `&mut` but `item` by value?

    > `self: &mut Library` because `add_item` needs to mutate the library's internal state (push into `items`) without taking ownership of the `Library` itself — the caller keeps using their `library` variable after the call returns; a `&mut` borrow is enough to modify it in place.
   > `item: Item` by value, on the other hand, because the library needs to *keep* the item forever, stored in its `Vec<Item>` — you can't store a borrowed reference there without giving `Library` a lifetime tied to wherever the caller's `Item` came from. Taking ownership is how the `Item` is handed off permanently.

5. When `add_item` returns `Err`, what happened to the `Item` the caller passed
   in? Was that a good design choice, and what is the alternative?

   > Because `item: Item` is taken by value, ownership moves into `add_item` at the call site — the caller has already lost direct access to it before the function body even runs. If `add_item` returns `Err`, the `Item` is simply dropped at the end of the function; its data is gone for good. For a *recoverable* error like a duplicate id, that's a questionable tradeoff — the caller may have done real work building that `Item` and now has to rebuild it from scratch just to retry with a different id. The alternative, used by `std::sync::mpsc::Sender::send(`Result<(), SendError<T>>`), is to hand the value back inside the error: something like `Result<(), (LibraryError, Item)>`, so a failed call returns both *why* it failed and the `Item` to retry with.

6. Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?

   > `find_item` only needs to answer "does an item with this id exist, and if so, what does it look like right now" — the `Item` itself never leaves `self.items`, so there's no reason to hand back an owned copy. Returning `Option<&Item>` borrows straight from `self` at zero cost: no cloning a `String` title/author just to answer a lookup. It's also the honest signature for what the data actually is — an `Item`'s state (especially `LoanStatus`) belongs to the `Library` and can change out from under a stale copy, so a borrowed reference, tied to `self`'s lifetime, is the only way to guarantee the caller is always looking at the library's current record rather than a snapshot that might already be wrong.


7. What is the lifetime `'a` in `items_by_author` actually saying?

   > The signature is `items_by_author<'a>(&'a self, author: &str) -> Vec<&'a Item>`. `'a` says the references inside the returned `Vec` are borrowed straight from `self`, and can't outlive that borrow — every `&Item` handed back points at an `Item` still living inside `self.items`, never a copy. With two reference parameters in play (`self` and `author`), the compiler can't guess on its own which one the output borrows from, so `'a` has to be written explicitly, tying the return value to `self` and not to `author`. This is what stops a caller from holding onto the returned items after the `Library` they came from has been dropped — the borrow checker rejects that at compile time as a lifetime error, rather than letting it become a dangling reference at runtime. `author` doesn't need a named lifetime because nothing in the output borrows from it; it's only compared against, then discarded.


8. Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the same
   `Library` at once, and how did you structure the method around that?

   > Both `items` and `members` are fields on the same `Library`. If you wrote helper methods like `find_item_mut(&mut self, id) -> Option<&mut Item>` and `find_member_mut(&mut self, id) -> Option<&mut Member>` and called both while keeping their results alive, it wouldn't compile — each call needs an exclusive `&mut self` borrow for as long as its returned reference lives, and the compiler can't see through the function boundary that one only touches `items` and the other only `members`; as far as borrowing is concerned, the first call occupies the whole `Library` until it's done being used, leaving no room for a second `&mut self` borrow.
   >
   > The fix isn't a workaround, it's realizing the two mutations never need to happen at the same instant: setting the item's status and pushing onto the member's borrowed list are independent writes, neither reading the other's value. So `checkout` validates first, using only immutable reads (unknown item, unknown member, lost, already on loan, borrow limit) that never hold a mutable borrow past the check. Only after validation passes does it mutate — get a `&mut Item`, make its one write, let that borrow end, then separately get a `&mut Member` and make its write. Because Rust's non-lexical lifetimes end a borrow at its last use rather than at the end of the block, the two mutable borrows never overlap, even though both happen inside the same `&mut self` call.

9. Why are `Library`'s fields private?

   >`Library`'s fields are private because `checkout`/`return_item` maintain an invariant across both of them at once: whenever an `Item`'s status is `OnLoan { member_id, .. }`, that member's `borrowed_item_ids` must contain the item's id, and vice versa — nothing in the type system enforces that on its own, it only holds because the library's methods update both sides together. If `items`/`members` were `pub`, outside code could reach in and set an item's status directly without touching the corresponding member's list, silently breaking that agreement — nothing would crash, the two records would just quietly disagree, and everything downstream that trusts the invariant (fee math, borrow-limit checks, returns) would be reasoning from bad data. Making the fields private closes that door: the only way to touch either field from outside the module is through the methods `Library` exposes, each written to keep the invariant intact. Privacy is what turns "the two lists stay in sync" from a comment into something actually enforced.


10. What duplication does the provided `late_fee_cents` remove, and what would
    you lose by making it a free function instead?

    > `late_fee_cents` is the one piece of arithmetic — "if `days_held` is within `loan_days()`, the fee is 0, otherwise charge `(days_held - loan_days()) * daily_late_fee_cents()`" — that both `impl LoanTerms for MediaKind` and `impl LoanTerms for Item` need identically; only the two inputs to that formula (`loan_days()`, `daily_late_fee_cents()`) actually differ per type. As a default trait method, that formula is written exactly once and every implementor inherits it just by supplying the two required methods, instead of each impl (and any future type implementing `LoanTerms`) duplicating the same fee math. A free function would remove the duplication too, but at a real cost: it loses the ergonomics of calling `.late_fee_cents(days_held)` directly on the value rather than the caller having to know the function exists and manually pull `loan_days()`/`daily_late_fee_cents()` out to pass in, and it loses the enforced link to `LoanTerms` itself — nothing would guarantee a same-named free function actually stays consistent with `loan_days`/`daily_late_fee_cents` the way a default trait method structurally must.


11. Why is `Result` preferable to `panic!` for validation failures? Name a
    place in this crate where a panic would be defensible.

   > A validation failure like an unknown item, an item already on loan, or a member at the borrow limit is an *expected* outcome — any real caller will hit these regularly, and it isn't a bug in the library when it happens. `Result` lets the caller see exactly which `LibraryError` occurred, with its ids, and decide how to respond; `panic!` would abort the whole program the instant someone tried to over-borrow, with no way for any caller to react. `panic!` is for the opposite situation: a violated internal invariant the code itself guaranteed, not bad input from outside.
   >
   > A defensible panic in this crate: inside `return_item`, after already confirming (and rejecting `ItemNotOnLoan` otherwise) that the item's status is `OnLoan`, pulling `day_borrowed` back out with a `match` that ends `_ => unreachable!("just confirmed this item is on loan")`. That branch should be impossible to hit — the status was just checked two lines above with no mutation in between — so if it ever fired, it would mean `return_item`'s own logic broke its own guarantee, not that a caller passed bad data. That's exactly the case `panic!`/`unreachable!` is for: failing loudly on a broken internal promise, rather than routing it through `Result` as if it were a normal, expected outcome.


12. Which derive did you deliberately leave off a type, and why?

   > I left `Clone` off `Item` and `Member`. Neither can derive `Copy` (both contain a `String`), but `Clone` would compile fine — `String` implements it — so leaving it off was a real choice, not a compiler restriction. `Library` is built around there being exactly one `Item` per book and one `Member` per person, which is the same reason its fields are private: every mutation has to funnel through `Library`'s own methods so an item's status and a member's borrowed list can't drift apart. `Clone` would open a second way to break that guarantee — cloning an `Item` produces a second value with the same id, frozen at whatever status it had at clone time; if the real item is later checked out, the clone still claims `Available` and quietly lies about the book's true state. Leaving off `Clone` means the only way to obtain an `Item`/`Member` at all is through a reference the `Library` hands out (`find_item`, `available_items`, etc.), so there's never an independent copy of a book or a person that can silently go stale.


### Ownership experiments (Part 7)

**Experiment A** — read `item.title` after `library.add_item(item)?`.

> error[E0382]: borrow of moved value: `item`
  --> src/main.rs:12:20
   |
 9 |     let item = Item::new(1, "Dune".into(), "Frank Herbert".into(), MediaKind::Book { pages: 320 });
   |         ---- move occurs because `item` has type `Item`, which does not implement the `Copy` trait
10 |
11 |     library.add_item(item)?;
   |                      ---- value moved here
12 |     println!("{}", item.title); // <- Experiment A
   |                    ^^^^^^^^^^ value borrowed here after move


> add_item(item: Item) takes ownership by value, so calling library.add_item(item) moves item out of main — it's not Copy, so there's no implicit duplicate left behind. By the time println! tries to read item.title, the binding is gone; the compiler catches this at compile time rather than letting it become a use-after-move bug at runtime.

**Experiment B** — hold the result of `library.find_item(1)`, call
`library.checkout(..)?`, then print what you held.

> error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
  --> src/main.rs:13:5
   |
12 |     let found = library.find_item(1);
   |                 ------- immutable borrow occurs here
13 |     library.checkout(1, 100, 0)?; // <- Experiment B
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
14 |     println!("{:?}", found);
   |                      ----- immutable borrow later used here


> find_item returns &Item borrowed from library, and found is still alive at the println! three lines later — so the compiler must keep that immutable borrow live across the whole span. checkout needs &mut library to update the item's status and the member's list, but Rust never allows a mutable borrow to coexist with a live immutable one, since the mutation could invalidate what found points to (e.g. if the Vec<Item> reallocated). The borrow checker rejects it at compile time instead of risking a dangling reference at runtime.

## Design notes

   > `checkout` and `return_item` both split their work into two phases: validate using only immutable borrows (`find_item`/`find_member`), then mutate using two separate `iter_mut().find(...)` calls, one for the item and one for the member, each ended before the next begins. This is what lets both mutations happen inside a single `&mut self` call without the borrow checker rejecting overlapping mutable borrows — see the write-up for question 8. Keeping the two data structures in agreement isn't enforced by the type system on its own; it's enforced by every path that changes one of them (`checkout`, `return_item`) always changing the other in the same call, and by `Library`'s fields being private so nothing outside these methods can touch either list independently (question 9).

   > The `.unwrap()` calls in both methods, re-finding the item/member by id after already validating their existence a few lines above, are a deliberate use of an infallible-in-practice operation rather than routing an impossible case through `Result`. Nothing between the initial `find_item`/`find_member` lookup and the later `iter_mut().find(...)` removes anything from `self.items`/`self.members`, so the second lookup cannot fail; if it ever did, that would indicate a bug in this method's own logic, not bad caller input — exactly the class of failure `panic!`/`.unwrap()` is appropriate for (question 11).

   > I did not attempt the optional Part 9 generic `filter_items`. `items_by_author` and `available_items` are both `self.items.iter().filter(...).collect()` with only the predicate differing, so a `fn filter_items(&self, predicate: impl Fn(&Item) -> bool) -> Vec<&Item>` would let both be re-expressed as one-liners calling it with a closure.


## Example output

Paste the output of `cargo run` here once Part 8 is complete.
> checked out: Item 1: "Dune" by Frank Herbert (book (320 pages)) - on loan to member 100 since day 5
>returned late, fee owed: 100 cents
>now: Item 1: "Dune" by Frank Herbert (book (320 pages)) - available
>handled error: item 1 is not currently on loan