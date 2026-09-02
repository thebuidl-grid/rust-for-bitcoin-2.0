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

## Part 7 — Ownership experiments

I uncommented each experiment one at a time in `ownership_experiments()`
(`src/main.rs`), ran `cargo check`, and pasted the real error below before
commenting the line back out.

### Experiment A — reading `item.title` after `library.add_item(item)?`

```
error[E0382]: borrow of moved value: `item`
 --> examples/experiment_a.rs:8:20
  |
5 |     let item = Item::new(1, "superstory".into(), "jide kosoko".into(), MediaKind::Book { pages: 412 });
  |         ---- move occurs because `item` has type `Item`, which does not implement the `Copy` trait
6 |
7 |     library.add_item(item).unwrap();
  |                      ---- value moved here
8 |     println!("{}", item.title);
  |                    ^^^^^^^^^^ value borrowed here after move
```

`add_item` takes `item` by value, so calling it hands the `Item` over to the
library for good — it gets pushed into `self.items` and the local `item`
binding is empty afterward. `Item` isn't `Copy` (it owns `String`s, and you
can't `Copy` heap data), so `item.title` on the next line is reading from a
binding that no longer has anything in it. This is the kind of bug that would
be a use-after-free or a silent double-owned string in C; here it just
doesn't compile.

### Experiment B — holding `library.find_item(1)` across a `checkout` call

```
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
  --> examples/experiment_b.rs:11:5
   |
10 |     let found = library.find_item(1);
   |                 ------- immutable borrow occurs here
11 |     library.checkout(1, 100, 0).unwrap();
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
12 |     println!("{:?}", found);
   |                      ----- immutable borrow later used here
```

`found` is a `&Item` borrowed from `library`, and it's still going to be used
later (in the `println!`), so that borrow spans both lines. `checkout` needs
`&mut self` to actually update the item's status, and Rust won't let a
mutable borrow open while a shared one is still in use — if it did, `found`
could end up pointing at an item whose status just changed underneath it.
The fix is just not to hold `found` across the call: read what you need from
it, or drop it, before calling `checkout`.

## Written answers

1. **Why is `LoanStatus` an enum rather than a `bool` plus two `Option`
   fields?** Because the three states are mutually exclusive, and an enum is
   the only way to make the compiler enforce that. If it were `is_lost: bool`
   plus `borrower: Option<u32>` plus `day_borrowed: Option<u32>`, nothing
   stops you from ending up with an item that's lost *and* on loan, or a
   `borrower` set with no `day_borrowed` to go with it. `OnLoan { member_id,
   day_borrowed }` keeps those two numbers glued together as one fact, and
   `Available`/`Lost` don't drag around fields that don't apply to them.

2. **What does `match` force you to do when a fourth `MediaKind` is added
   later?** Every `match` on `MediaKind` that doesn't have a wildcard arm
   (`loan_days`, `daily_late_fee_cents`, the `Display` impl) stops compiling
   the moment the new variant shows up, until you add an arm for it. So
   instead of shipping a bug where the new media type quietly falls through
   with the wrong loan length, you get a build error pointing at every spot
   that needs updating.

3. **`Item::new` takes `String` rather than `&str`. Who owns the title
   afterwards?** The `Item` does. Handing it an owned `String` means the
   title moves in and belongs to the struct from then on — the `Item` isn't
   tied to whatever variable or literal the caller built the string from, and
   `Item` doesn't need a lifetime parameter to hold onto it.

4. **Why does `add_item` take `self` by `&mut` but `item` by value?**
   `&mut self` because all it's doing is pushing onto a `Vec` that already
   belongs to the library — no reason to take ownership of the whole
   `Library` just to add one item. `item` by value because the opposite is
   true for the item itself: the library needs to own it indefinitely once
   it's stocked, and a borrowed `&Item` can't be stored past the call without
   tying `Library`'s lifetime to whatever stack frame the caller built the
   item in.

5. **When `add_item` returns `Err`, what happened to the `Item` the caller
   passed in? Was that a good design choice, and what is the alternative?**
   It's dropped. The item moved into the function, validation failed, the
   function returned early, and there was nowhere else for it to go. That's
   fine here since neither `EmptyTitle` nor `DuplicateItemId` is something
   you'd retry with the exact same value anyway. Where it would matter is if
   the caller wanted to fix something small and resubmit without rebuilding
   the whole `Item` — then you'd want the error to hand the value back, e.g.
   `Err(LibraryError::EmptyTitle { item })`.

6. **Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?**
   Returning `Option<Item>` would mean cloning it out (`Item` isn't `Copy`),
   and moving it out of the `Vec` isn't an option either without leaving a
   hole. A reference lets the caller look without taking, and it costs the
   same no matter how big `Item` gets.

7. **What is the lifetime `'a` in `items_by_author` actually saying?** That
   the `Vec<&'a Item>` coming back can't outlive the `&'a self` it came from.
   Every reference in that vector points straight into `library`'s own
   `items` vector, so as soon as `library` goes away (or gets borrowed
   mutably somewhere else), those references have to go away too. It's the
   compiler tying the output's lifetime to the input borrow's.

8. **Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the
   same `Library` at once, and how did you structure the method around
   that?** Two different fields being borrowed mutably at once (`items` and
   `members`) wouldn't actually be a problem on its own — the trouble is
   that `checkout` needs to *read* both to run its five checks before it
   *writes* either, and `find_item`/`find_member` borrow the whole
   `Library` through `&self`. So I split the method into two passes:
   everything that only needs to read runs first, through short-lived shared
   borrows that each end as soon as the check is done, and only once every
   check has passed does the method reach for `iter_mut()` — first on
   `items`, then on `members`, one after the other, never both borrowed at
   the same time. Validate first, mutate second, and never hold more than
   one borrow open at once.

9. **Why are `Library`'s fields private?** So there's exactly one door in:
   `checkout` and `return_item` are the only code paths that can touch an
   item's `status` or a member's `borrowed_item_ids`, and both of them always
   update the two together. If `items` and `members` were public, it'd be
   possible to mark an item `OnLoan` without ever touching the borrower's
   list, and the two would drift apart with nothing to catch it.

10. **What duplication does the provided `late_fee_cents` remove, and what
    would you lose by making it a free function instead?** Without it, both
    `impl LoanTerms for MediaKind` and `impl LoanTerms for Item` would each
    need their own copy of `days_held.saturating_sub(self.loan_days()) *
    self.daily_late_fee_cents()`, and every future implementor would have to
    copy it again. As a default trait method it's written once and every
    implementor gets it for free. A free function would remove the
    duplication too, but it wouldn't be reachable as `.late_fee_cents(...)`
    on the value — anyone holding a `&dyn LoanTerms` would have to know to
    call it separately — and nothing would stop a future impl from
    overriding `loan_days`/`daily_late_fee_cents` while forgetting the
    formula exists at all.

11. **Why is `Result` preferable to `panic!` for validation failures? Name a
    place in this crate where a panic would be defensible.** The failures
    this crate validates for — an empty title, an id nobody registered, a
    return day earlier than the borrow day — are ordinary things a caller can
    run into just by using the library normally, not proof something is
    broken. `panic!` ends the program and gives the caller no way to react;
    `Result` puts the failure in the signature, so the compiler makes every
    caller deal with it one way or another. A panic earns its keep where the
    thing that broke is the library's own bookkeeping, not the caller's
    input — the `.expect(...)` calls inside `checkout`/`return_item`, right
    after `find_item`/`find_member` have already confirmed an id exists, fall
    into that bucket. If that lookup ever came back empty there, it wouldn't
    mean the caller did anything wrong — it'd mean this crate's own
    invariants broke.

12. **Which derive did you deliberately leave off a type, and why?** `Clone`
    (and `Copy`) off both `Item` and `Member`. They're each meant to have one
    owner — the `Library` — and everything else is supposed to go through
    borrows like `find_item` and `items_by_author` instead of copies. Adding
    `Clone` would make it too easy to end up with a second, independent
    `Item` whose `status` quietly stops matching the one actually sitting in
    the library, which is exactly the drift that keeping the fields private
    is meant to prevent.

## Design notes

The one thing I cared about most was keeping an item's `LoanStatus` and its
borrower's `borrowed_item_ids` from ever telling two different stories. That
comes down to `items` and `members` being private, so the only way in is
`checkout` and `return_item`, and both of them always update the item and
the member together, and only after every check has already passed
(validate first, mutate second — question 8 above goes into why). There's no
path through this crate that can update one side without the other.

`checkout`'s five checks run in the order `ASSIGNMENT.md` lays out — unknown
item, unknown member, lost, already on loan, borrow limit — each bailing out
with its own `return Err(...)` (or `?`), so fixing one problem always
surfaces the next real one instead of some later, unrelated error.

For `late_fee_cents`, I used `saturating_sub` instead of plain `-` for
`days_held - loan_days`, since an on-time or early return means
`days_held <= loan_days`, and `u32` subtraction panics on underflow in debug
builds. `saturating_sub` just gives zero overdue days in that case instead,
which is what you want anyway.

I also went ahead and did the optional Part 9. `Library::filter_items` takes
any `Fn(&Item) -> bool` and hands back matching references without cloning
anything, and `items_by_author`/`available_items` are now just one-liners
built on top of it instead of each rolling its own
`iter().filter().collect()`.

## Example output

```
Stocked the library:
  #1 "superstory" by jide kosoko [book (412 pages)] — available
  #2 "things fall apart" by chinua achebe [audiobook (970 minutes)] — available

Ada checked out item 1: #1 "superstory" by jide kosoko [book (412 pages)] — on loan to member 100 since day 10
Ada returned item 1 on day 40 and owes 225 cents in late fees.

handled error: no member with id 999 was found
```
