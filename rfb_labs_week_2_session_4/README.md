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

## Ownership Expermient

I already fixed the errors after implementing part 6 before seeing part 7 was about it, so i have no error message.


Answer in your own words. Add both ownership compiler errors from Part 7 as
fenced text blocks, then explain what caused each.

1. Why is `LoanStatus` an enum rather than a `bool` plus two `Option` fields?

Ans: Because an enum can only be of it's variant at a time, and an enum makes that a rule the compiler enforces — a bool + two Options only makes it a rule you have to remember.

2. What does `match` force you to do when a fourth `MediaKind` is added later?

Ans: Because match in Rust must be exhaustive — every variant has to be covered (urm). So if someone adds a fourth variant, say:

```bash
enum MediaKind {
    Book { pages: u32 },
    Audiobook { minutes: u32 },
    Ebook { size_kb: u32 },                                                                                     
    Magazine { pages: u32 },   // new
}                                                                                              
```                                                                         
then every existing match self { ... } on MediaKind in this file — loan_days(), daily_late_fee_cents(), the Display impl — stops compiling with a "non-exhaustive match" error until you add a Magazine arm to each one.

3. `Item::new` takes `String` rather than `&str`. Who owns the title afterwards?

Ans: Because the Item itself owns the title — the String is moved into the struct, no borrowing involved.

4. Why does `add_item` take `self` by `&mut` but `item` by value?

Ans: self only needs to be modified (push into its Vec), but item needs to be moved in and stored — those are different needs, so they take different parameter kinds.

5. When `add_item` returns `Err`, what happened to the `Item` the caller passed
   in? Was that a good design choice, and what is the alternative?

Ans: The item is gone — dropped. add_item took ownership of item by value, and on the Err paths it never puts item anywhere (not pushed into self.items, not returned to the caller), so when the function returns, item simply falls out of scope and is destroyed. 
The alternative: hand the item back on failure, so ownership returns to the caller when the operation didn't succeed.
The general principle: if a function consumes a value and can fail in a way the value survives, it's often kinder to give that value back on the failure path — otherwise "by value" quietly becomes "destroyed on any error," which surprises callers who expected to retry.


6. Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?

Ans: returning Option<Item> would require moving (or cloning) the item out of self.items, but find_item only has a shared reference (&self) — it doesn't own the items, so it can't hand out ownership of one.

7. What is the lifetime `'a` in `items_by_author` actually saying?

Ans: it's saying "the references in the returned Vec are only valid for as long as the Library they came from is still borrowed" — it ties the output's lifespan to the input's.

8. Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the same
   `Library` at once, and how did you structure the method around that?

Ans: Each of those methods takes &mut self — the whole Library, not just one field. The borrow checker can't look inside the function body to see that find_item_mut only ever touches self.items; from the caller's point of view, the first call borrows all of self mutably for as long as item is alive, so a second &mut self call (find_member_mut) while item is still live is a conflict, even though the two methods never actually touch the same data.

`checkout` is structured around this: it avoids the method-call trap by reaching into self.items and self.members directly with .iter_mut().find(...) inline, in the same function body, rather than delegating to two separate &mut self-taking helpers. That keeps the two mutable borrows visibly disjoint to the compiler, so it can grant both at once.

9. Why are `Library`'s fields private?

Ans: So outside code can't touch items/members directly and break the invariant that an item's LoanStatus and a member's borrowed_item_ids always agree with each other. By making the fields private, the only way to read or change them is through Library's own methods (find_item, find_member, checkout, return_item, etc.) — and those methods are the ones responsible for keeping both sides consistent.

10. What duplication does the provided `late_fee_cents` remove, and what would
    you lose by making it a free function instead?

Ans: Without it, every type that implements LoanTerms (right now MediaKind and Item, and anything added later) would have to write its own version of "fee = rate × days held" instead, the trait provides one default implementation.

11. Why is `Result` preferable to `panic!` for validation failures? Name a
    place in this crate where a panic would be defensible.

Ans: a panic! aborts the whole program — the caller gets no chance to recover, log the problem, ask the user to fix their input, or retry. Validation failures like "duplicate id," "empty title," "item not found," "borrow limit reached" are all expected, recoverable outcomes of normal usage — a caller passing a bad id isn't a bug in the library, it's just something that happens. Result lets the caller decide what to do about it (retry, report, ignore) instead of the library unilaterally deciding "this is fatal.

12. Which derive did you deliberately leave off a type, and why?

Ans: Clone on Item (and Member, for the same reason).MediaKind and LoanStatus are small, self-contained value types — copying/cloning them is cheap and there's no ownership story to protect (there's only ever "a LoanStatus," not "the LoanStatus that lives in the library"). Item anare the things the Library is the single source of truth for.Leaving Clone off Item is what forces find_item, items_by_author, etc.to return &Item instead of being able to cheaply return Option<Item> / Vec<Item> by cloning.

## Design notes

Describe any choices you made, including how you kept an item's status and its
borrower's list from drifting apart, and (if attempted) the optional generic
search.


Ans: Private fields on Library — items/members can only change through checkout/return_item, which always update both sides in the same call, so status and borrowed-list can't go out of sync. Also attempted the optional generic search: filter_items takes a Fn(&Item) -> bool predicate, and items_by_author/available_items are both re-expressed as one-line calls to it.

## Example output

Paste the output of `cargo run` here once Part 8 is complete.
```bash
Item 1: Item {
    id: 1,
    title: "Things Fall Apart",
    author: "Chinua Achebe",
    kind: Book {
        pages: 1000,
    },
    status: Available,
}
On-time return fee: 0 cents
Late return fee: 225 cents
Handled error: ItemAlreadyOnLoan: The item id: 2 has being loaned to member id: 7
```
