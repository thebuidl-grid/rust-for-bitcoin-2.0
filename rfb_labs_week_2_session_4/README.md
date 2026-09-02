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

## Status

All nine parts are implemented, including the optional Part 9. Every starter
test has had its `#[ignore]` removed and the remaining required cases are
written, for 28 passing tests and nothing ignored.

```text
cargo test     28 passed; 0 failed; 0 ignored
cargo fmt --check     clean
cargo clippy --all-targets --all-features -- -D warnings     clean
```

The two ownership experiments live in
[examples/ownership_experiments.rs](examples/ownership_experiments.rs), with
the failing lines commented out and the version that compiles kept beside each
one.

## Written answers

### Part 7 — the two ownership experiments

**A** — reading `item.title` after `library.add_item(item)?`:

```text
error[E0382]: borrow of moved value: `item`
  --> examples/ownership_experiments.rs:27:28
   |
18 |     let item = Item::new(
   |         ---- move occurs because `item` has type `Item`, which does not implement the `Copy` trait
...
25 |     library.add_item(item)?;
   |                      ---- value moved here
26 |
27 |     println!("stocked {}", item.title);
   |                            ^^^^^^^^^^ value borrowed here after move
   |
   = note: this error originates in the macro `$crate::format_args_nl` which comes from the expansion of the macro `println` (in Nightly builds, run with -Z macro-backtrace for more info)
```

`add_item` takes its parameter by value, so the call *moves* the `Item` into
the library. `Item` contains two `String`s, which own heap allocations, so it
cannot be `Copy` — there is exactly one owner of that title at any moment, and
after the call it is the `Library`. The local `item` binding is not merely
stale, it is dead: the compiler will not let me read even a field of it. What
makes this a good error rather than an annoying one is that the alternative in
a language without move semantics is a second variable silently aliasing data
someone else can now mutate. The fix is to ask the new owner: `library.find_item(1)`.

**B** — holding `library.find_item(1)` across `library.checkout(..)?`:

```text
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
  --> examples/ownership_experiments.rs:46:5
   |
44 |     let held = library.find_item(1);
   |                ------- immutable borrow occurs here
45 |
46 |     library.checkout(1, 100, 5)?;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
47 |
48 |     println!("held {held:?}");
   |                     ---- immutable borrow later used here
```

`find_item` returns an `Option<&Item>` borrowed from the library, so as long as
`held` is alive the whole `library` is immutably borrowed. `checkout` wants
`&mut self`, and Rust will not hand out a mutable borrow while a shared one is
outstanding. The last line is the reason the error exists at all: with
non-lexical lifetimes the shared borrow would have ended right after line 44 if
nothing used it later, and the code would compile. Because line 48 reads
`held`, the borrow has to stretch across the mutation.

This is not pedantry — `checkout` writes through `self.items[..]`, and `Vec`
mutation is exactly the case where a retained reference could dangle if the
backing buffer were reallocated. The fix is to shorten the borrow: copy out the
few `Copy` fields needed, or simply look the item up again after the mutation.

### The twelve questions

**1. Why is `LoanStatus` an enum rather than a `bool` plus two `Option`
fields?**

Because `on_loan: bool`, `borrower: Option<u32>`, `day_borrowed: Option<u32>`
can represent eight combinations, of which only a couple are meaningful. It
lets me build `on_loan: true` with `borrower: None`, or a borrower with no
borrow day, and nothing objects until something reads that state at runtime.
The enum makes those states unrepresentable: a `LoanStatus::OnLoan` *always*
carries both a `member_id` and a `day_borrowed`, because the variant owns them.

The flag version also has no room for `Lost` — a lost item is neither available
nor on loan, so a third bool would appear, and now sixteen combinations. And
`return_item` would have to `unwrap()` two `Option`s that it "knows" are `Some`,
which is precisely the kind of knowledge that stops being true after someone
edits a different file. In the enum version the data comes out of the `match`
arm already proven to exist.

**2. What does `match` force you to do when a fourth `MediaKind` is added
later?**

It forces me to visit every site that reasons about media kinds, because a
non-exhaustive `match` is a compile error, not a warning. Adding
`MediaKind::Periodical` would break `loan_days`, `daily_late_fee_cents`, and
`Display` — the three places that genuinely need a decision — and the compiler
lists them for me. That is the whole trade: I accept a compile error today in
exchange for never shipping a media kind that silently inherited some other
kind's loan length from an `_ =>` fallback.

This is also why I avoided catch-all arms in the two `LoanTerms` impls. In
`daily_late_fee_cents` I wrote `Book { .. } | Audiobook { .. } => 25` rather
than `_ => 25`, so a new variant still fails to compile. The one place a
wildcard is fine is a `..` inside a variant pattern, which ignores *fields*, not
variants.

**3. `Item::new` takes `String` rather than `&str`. Who owns the title
afterwards?**

The `Item` does, and after `add_item` the `Library` owns it transitively, since
it owns the `Item`. Taking `String` means the caller decides how the string was
produced — a literal via `.into()`, a formatted name, a line read from a file —
and hands over the allocation rather than forcing a copy the caller may not
need. Taking `&str` would mean `Item::new` allocates internally on every call
and the caller keeps a copy nobody asked for. It would also drag a lifetime
parameter onto `Item` if I tried to store the borrow instead, and
`Library` could then never outlive whatever buffer the titles came from.

**4. Why does `add_item` take `self` by `&mut` but `item` by value?**

Two different relationships. The library already exists and outlives the call;
I only need to modify it temporarily, so `&mut self` borrows it and gives it
back. The item, by contrast, has to be *kept* — it is pushed into a `Vec` that
lives as long as the library. You cannot store a borrow that outlives the
borrow, so the only way to stock the item is to own it. The signature is the
API's honest statement of that: "lend me the library, give me the item."

**5. When `add_item` returns `Err`, what happened to the `Item` the caller
passed in? Was that a good design choice, and what is the alternative?**

It was moved into `add_item` and dropped when the function returned, taking its
two `String` allocations with it. The caller has already lost the binding to the
move, so a rejected item is simply gone — to retry with a corrected title they
must rebuild it from scratch.

For this crate I think that is the right call, because both rejections are
programmer errors rather than recoverable conditions: an empty title or a
duplicate id means the calling code is wrong, and the fix is a code change, not
a retry. The alternative is to hand the value back on failure, either as
`Result<(), (Item, LibraryError)>` or by putting the `Item` inside the error
variants. That is the standard move wherever the rejected value is worth
recovering: `String::from_utf8` hands the original bytes back through
`FromUtf8Error::into_bytes`, and `mpsc::SendError` carries the message that
could not be sent. The cost is that the error type is no longer small, cheap and
`Copy`-ish; it becomes as large as an `Item`, every `Result` in the crate widens
to hold it, and the tests can no longer compare errors with a plain `assert_eq!`
against a literal. Given the failures here are not retryable, that price buys
nothing.

**6. Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?**

Because `Option<Item>` would have to move the item out of the library, which is
impossible through `&self` and would be wrong anyway — a lookup should not empty
the shelf. The only way to satisfy that signature is to `clone`, and a clone is
worse than slow here: it is a second `Item` with its own `status` field, which
starts out equal to the library's copy and drifts the instant anything is
checked out. `searching_by_author_borrows_rather_than_clones` pins this down
with `std::ptr::eq`, asserting the search result is the *same object* as the
library's, not an equal one.

**7. What is the lifetime `'a` in `items_by_author` actually saying?**

That the `&Item`s in the returned `Vec` borrow from `self`, and therefore the
library must outlive the results. Note what it does *not* say: nothing ties the
output to `author`. The author string is only read during the call and can be a
temporary that dies immediately afterwards.

Elision would infer the same thing — with a `&self` parameter, the third
elision rule gives every elided output lifetime the lifetime of `self`, so
removing `'a` compiles identically. So `'a` here is documentation rather than
necessity; it makes explicit which of the two input references the results are
tied to, which is the question a reader actually has.

**8. Why can't `checkout` hold a `&mut Item` and a `&mut Member` from the same
`Library` at once, and how did you structure the method around that?**

The precise reason is that each would come from a method call, and a method
taking `&mut self` borrows the *entire* `Library`, not the field it touches. So
a hypothetical `find_item_mut(..)` and `find_member_mut(..)` produce two
overlapping mutable borrows of the same value, which is exactly what the
aliasing rule forbids. It is worth being exact here, because direct field access
inside one function body — `&mut self.items[i]` and `&mut self.members[j]` —
*does* compile: the borrow checker sees disjoint fields. It is the abstraction
boundary of a method, not the struct itself, that collapses the two borrows into
one.

I structured around it by never holding either reference. The private helpers
`item_index` and `member_index` return `Option<usize>`, and a `usize` borrows
nothing, so each lookup's borrow ends on the line it started. Validation then
reads through those indices, and the two mutations happen as separate indexed
statements after the last possible failure. That ordering is what makes the
"both or neither" guarantee real, and it fell out of the borrow rule rather than
fighting it.

**9. Why are `Library`'s fields private?**

Because the library's one real invariant spans both of them: an item whose
status is `OnLoan { member_id }` must appear in that member's
`borrowed_item_ids`, and nothing else may. Public `Vec` fields would let any
caller push an id onto a member's list without touching the item, or mark an
item `Available` while a member still holds its id, and no amount of care inside
`checkout` could prevent it. Privacy makes `checkout` and `return_item` the only
two writers in the program, so there are exactly two places to audit.

**10. What duplication does the provided `late_fee_cents` remove, and what
would you lose by making it a free function instead?**

It removes the `max(0, days_held - loan_days) * daily_fee` formula from both
impls — the saturating subtraction, the multiplication, and the on-time case
that owes nothing. Without the default method, `MediaKind` and `Item` would each
carry their own copy of that arithmetic, and the `Item` one would be a
transcription of the `MediaKind` one, which is precisely how a fee policy ends
up applied inconsistently.

As a free function `late_fee_cents(terms: &impl LoanTerms, days: u32)` the
arithmetic would still be shared, so what I would lose is not deduplication but
three other things. It could no longer be called as `item.late_fee_cents(30)`,
which is how the call site reads best. It would stop being part of the trait
contract, so a future implementor of `LoanTerms` would get no fee behaviour at
all unless they remembered the helper existed. And it could not be overridden —
a `ReferenceOnly` kind with a flat non-return charge can replace the default
method today, whereas a free function is take-it-or-leave-it.

**11. Why is `Result` preferable to `panic!` for validation failures? Name a
place in this crate where a panic would be defensible.**

Because none of these failures are bugs. A member reaching their borrow limit or
an item already being out is the system working correctly, and the caller — a
counter clerk's UI, say — needs to show a message and carry on. A panic unwinds
the thread and takes the whole session with it, and it is invisible in the type
system, so nothing warns a caller that a call site needs handling. `Result` puts
the failure in the signature where the compiler enforces a decision, and the
ordered variants let a caller fixing one problem at a time predict the next
error.

The defensible panic is already in the crate: `self.items[item_index]` in
`checkout` and `return_item`. That index came from `position` a few lines
earlier and the vector has not been touched in between, so an out-of-range index
would mean the library's own invariant is broken. That is not a condition a
caller can handle or has any business seeing as a `Result` — continuing on
corrupt state is worse than stopping. Same reasoning for the `.unwrap()` calls
in the tests: there, a failure is the test failing, which is the point.

**12. Which derive did you deliberately leave off a type, and why?**

`Clone`, on `Item`, `Member`, and `Library` alike — and leaving it off was a
decision, not an omission, since `#[derive(Clone)]` compiles fine on all three.
The reason is the same one behind question 6. A cloned `Item` is a detached copy
of a `status` field the library believes it alone controls; a cloned `Member`
duplicates a `borrowed_item_ids` list that is supposed to be the single record
of what someone holds. Making a duplicate cheap and one keyword away invites
exactly the drift that private fields and borrowing lookups were put there to
prevent. If a caller genuinely needs an owned snapshot — for a report, say —
that should be a deliberate, differently-named conversion, not a reflex.

I also left `Copy` off `LibraryError` (it holds only integers, so it could be
`Copy`), because errors are usually moved once into a `?` and a `Copy` error is
easy to accidentally use twice. `LoanStatus` and `MediaKind`, by contrast, keep
`Copy`: being able to pull a status out of an item by value is what ends the
borrow in `return_item` early enough for the mutation to follow.

## Design notes

**Keeping status and the borrowed list in agreement.** The rule I settled on is
that every fallible check happens before any mutation, and once the first write
lands nothing between it and the last write may fail. In `checkout` that means
all five validations run first, in the documented order, and only then do the
two writes happen back to back with no `?` between them. `return_item` is the
same shape: the loan details come out of the status, the `checked_sub` that
could fail is done *before* the status is reset, and the fee is computed while
the item is still on loan. Combined with private fields, this leaves exactly two
functions in the program that can write to either collection, and neither can
stop halfway. Two tests exist to hold that line — one checks a failed
double-checkout leaves the second member holding nothing, and one checks that a
rejected early return leaves both the status and the list untouched.

**Indices rather than references.** The `item_index` / `member_index` helpers
exist because a `usize` borrows nothing. They are what let validation and
mutation live in the same function without the two-mutable-borrows problem in
question 8, and they keep the method flat instead of pushing the mutation into a
nested scope.

**Validation order as an API promise.** The order in `checkout` is not
arbitrary, and `checkout_reports_the_unknown_item_before_the_unknown_member`
tests it with both ids wrong so the ordering itself is what is being asserted.
Same for lost-before-on-loan.

**Fee arithmetic.** `saturating_sub` then `saturating_mul` in the shared
`late_fee_cents` covers the on-time case, since a loan returned early or on time
saturates to zero days overdue and no fee. Ebooks fall out of the same formula
with a zero daily rate rather than needing a special case, which is why an ebook
kept 100 days past its 7-day loan still owes nothing without a single `if`.

**The optional generic search (Part 9).** `filter_items` takes
`F: Fn(&Item) -> bool` and both `items_by_author` and `available_items` are now
one line each on top of it. `Fn` rather than `FnMut` or `FnOnce` because the
closure is called once per item and has no reason to hold mutable state; the
tighter bound is the honest one and keeps the search side-effect free. It
returns `Vec<&Item>` for the same reason the other lookups do — the closure only
ever sees a shared reference, so no predicate can mutate the catalogue it is
filtering. It also composes: the test filters on `loan_days() >= 14`, a query
neither named lookup provides.

**A note on the `Lost` status.** Nothing in the public API marks an item lost,
but `Item`'s fields are public, so the tests stock an already-lost item
directly. If this grew, `report_lost` would belong on `Library` next to the
other two writers, since losing an item also has to clear it from its borrower's
list — the same invariant, and the same reason it cannot be a field assignment.

## Example output

```text
== catalogue ==
  #1 "Dune" by Frank Herbert (book, 320 pages) — available  (may be kept 21 days)
  #2 "Children of Dune" by Frank Herbert (book, 180 pages) — available  (may be kept 21 days)
  #3 "Project Hail Mary" by Andy Weir (audiobook, 540 minutes) — available  (may be kept 14 days)
  #4 "The Rust Programming Language" by Steve Klabnik (ebook, 1200 kB) — available  (may be kept 7 days)

longest loan: Dune at 21 days

== loan ==
  #1 "Dune" by Frank Herbert (book, 320 pages) — on loan to member 100 since day 10
  Ada now holds [1]

== late return on day 40 ==
  30 days held against a 21 day loan — 9 days overdue, $2.25 owed
  #1 "Dune" by Frank Herbert (book, 320 pages) — available

== a handled error ==
  could not return item 2: item 2 is not on loan, so it cannot be returned
```
