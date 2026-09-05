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

### Part 7, Experiment A — reading a moved value

Reading `item.title` after `library.add_item(item)?`:

```text
error[E0382]: borrow of moved value: `item`
  --> src/main.rs:16:32
   |
 9 |     let item = Item::new(
   |         ---- move occurs because `item` has type `Item`, which does not implement the `Copy` trait
...
15 |     library.add_item(item)?;
   |                      ---- value moved here
16 |     println!("still mine? {}", item.title);
   |                                ^^^^^^^^^^ value borrowed here after move
```

`add_item` takes `item: Item` **by value**, so calling it moves the item out of
`main` and into the library, which then pushes it into `self.items`. `Item`
owns two heap-allocated `String`s and so cannot be `Copy` — a bitwise copy
would leave two owners of the same buffer and a double free at drop. After the
move the local name `item` is dead, and reading `item.title` is a use-after-move
that the compiler rejects.

Ways to make it compile, in order of preference: print the title *before* the
call; borrow it back afterwards with `library.find_item(1)`; or clone the title
first if a genuinely independent copy is needed.

### Part 7, Experiment B — mutating while a borrow is alive

Holding `library.find_item(1)` across `library.checkout(..)?`:

```text
error[E0502]: cannot borrow `library` as mutable because it is also borrowed as immutable
  --> src/main.rs:21:5
   |
20 |     let held = library.find_item(1);
   |                ------- immutable borrow occurs here
21 |     library.checkout(1, 100, 0)?;
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
22 |     println!("held: {held:?}");
   |                      ---- immutable borrow later used here
```

`find_item` returns `Option<&Item>`, a reference *into* `library`, so `held`
keeps a shared borrow of the whole library alive. `checkout` needs `&mut self`,
and Rust forbids a mutable borrow while any shared borrow is live. The third
line is what makes this fatal: non-lexical lifetimes end a borrow at its last
use, so deleting the `println!` would compile — the error exists precisely
because `held` is used *after* the mutation.

This is not a technicality. `checkout` may push onto `self.items`, and a `Vec`
that reallocates moves its elements, leaving `held` dangling. The rule that
looks inconvenient here is the one preventing a use-after-free.

The fix is to not hold the borrow: re-query with `library.find_item(1)` after
the checkout, or copy out the scalars needed before mutating. That is exactly
the read-phase/write-phase structure `checkout` and `return_item` use
internally, for the same reason.

### 1. Why is `LoanStatus` an enum rather than a `bool` plus two `Option` fields?

Because it makes the impossible states unrepresentable. `is_on_loan: bool` plus
`borrower: Option<u32>` plus `day_borrowed: Option<u32>` can express eight
combinations, of which only two are meaningful. Nothing stops
`is_on_loan: true` with `borrower: None`, and every reader of that struct has
to remember which combinations are legal. The enum has exactly three states,
and `OnLoan` carries its two fields *inside* the variant, so the borrower and
the borrow day are either both present or both absent — never one without the
other.

`Lost` makes it worse for the flag design: it needs a second bool, and now
`is_on_loan && is_lost` is another nonsense state to defend against.

The payoff shows up in `return_item`. Matching `OnLoan { member_id,
day_borrowed }` hands over both values already proven to exist. The flag
version would need `borrower.unwrap()` — a panic waiting for the day someone
sets the bool and forgets the field.

### 2. What does `match` force you to do when a fourth `MediaKind` is added later?

It forces a decision at every site, and refuses to compile until it gets one.
None of this crate's matches on `MediaKind` has a `_` arm, so adding a
`Periodical` variant produces a non-exhaustive-patterns error in
`MediaKind::loan_days`, `MediaKind::daily_late_fee_cents`, and `Display for
MediaKind`. The compiler lists them; you work through the list.

That is the feature. A `_ => 14` catch-all would compile happily and quietly
give periodicals a two-week loan and a 25-cent fee that nobody chose. The
exhaustive match converts "we forgot to update the fee table" from a bug found
in production into a build failure.

### 3. `Item::new` takes `String` rather than `&str`. Who owns the title afterwards?

The caller hands ownership over and cannot use their `String` again. `Item`
owns the title from then on; once the item is stocked, the `Library` owns the
`Item` and so transitively owns the title, and the heap buffer is freed when
the item is removed or the library is dropped.

Taking `&str` would mean an internal `.to_string()` — the same allocation, just
hidden from the caller, and forced even on a caller who already had a `String`
to give away. Taking `String` makes the transfer visible at the call site:
`"Dune".into()` says plainly that an allocation happens here.

### 4. Why does `add_item` take `self` by `&mut` but `item` by value?

The two parameters have genuinely different fates, and the signature says so.
The library is *modified* and must outlive the call — the caller keeps using it
afterwards — so it is borrowed mutably. The item is *stored*: the library keeps
it after the call returns, so it cannot be a borrow without risking a dangling
reference to something the caller later drops. Storage is a transfer of
ownership, and by-value is how you spell that.

Read as documentation: `&mut self` promises "I will change this and give it
back", while `item: Item` announces "this is mine now."

### 5. When `add_item` returns `Err`, what happened to the `Item` the caller passed in?

It was destroyed. The item was moved into `add_item`, the early `return Err`
skipped the `push`, and the item was dropped at the end of the function. The
caller's value is gone whether the call succeeded or failed.

For `EmptyTitle` that is a poor outcome. The caller most likely wants to fix the
title and try again, and instead has to reconstruct the whole item from
scratch — including data they may no longer have.

The alternative is to hand the item back with the error, either as
`Result<(), (Item, LibraryError)>` or with a dedicated error type carrying the
item. The standard library does exactly this where it matters:
`std::sync::mpsc::SendError<T>` returns the undelivered message. The cost is a
noisier signature and a tuple every caller has to destructure, including the
majority who only want to know whether it worked. For an exercise this size the
simple signature is defensible; for a real API where items are expensive to
build, returning it would be the better trade.

### 6. Why does `find_item` return `Option<&Item>` rather than `Option<Item>`?

Two reasons, one cheap and one important.

The cheap one: `Item` is not `Copy`, so returning it by value means cloning two
`String`s on every lookup — an allocation for a read.

The important one: a clone is a *detached copy*. Write to it and the library
never hears about it; read from it later and you may be looking at a stale
title while the real item has moved on. A borrow is a view of the actual entry,
and the compiler guarantees the library outlives it.

There is also a mechanical point: `Library` could not return `Option<Item>`
without either removing the item from the catalogue or cloning it, and `Item`
does not even derive `Clone` (see question 12).

### 7. What is the lifetime `'a` in `items_by_author` actually saying?

That the returned references borrow from **the library**, and their validity is
bounded by the library's — not by the `author` string.

Concretely, this is fine:

```rust
let found = library.items_by_author(&String::from("Frank Herbert"));
println!("{}", found[0].title); // the temporary author string is long gone
```

I checked what the compiler does without the annotation, and it compiles: the
third elision rule says that when a method takes `&self`, `self`'s lifetime is
assigned to every output lifetime. So `'a` here is documentation of what
elision already infers, which is why the same signature written as a free
function is rejected —

```text
error[E0106]: missing lifetime specifier
  = help: this function's return type contains a borrowed value, but the
    signature does not say whether it is borrowed from `items` or `author`
```

— with two input references and no `&self`, there is no rule to pick between
them and you must say which one you meant.

### 8. Why can't `checkout` hold a `&mut Item` and a `&mut Member` at once?

Because both would be obtained through `&mut self`, and two simultaneous
mutable borrows of `*self` is E0499. The borrow checker reasons at the level of
the *method call*: a method taking `&mut self` may touch any field, so once one
lookup has borrowed the whole library mutably, nothing else can.

Worth noting the borrow checker is not being crude about it — writing
`&mut self.items` and `&mut self.members` in the same scope is perfectly legal,
because those are disjoint fields. It is routing through helper *methods* that
throws that information away.

I structured both `checkout` and `return_item` in two phases:

- **Read phase.** Locate the item and the member with `.position()`, which
  returns `usize` rather than a reference. A number is not a borrow, so the
  shared borrows end immediately and nothing is held across the boundary. All
  validation happens here, in the documented order, reading through shared
  borrows.
- **Write phase.** Index the two vectors directly, one statement each. The
  first mutable borrow ends at its semicolon, so the second is free to begin.

The phase split also buys atomicity: because no field is touched until every
check has passed, a rejected checkout leaves the library exactly as it was.

### 9. Why are `Library`'s fields private?

Because the invariant the library exists to maintain spans both of them: an
item whose status is `OnLoan { member_id }` must appear in that member's
`borrowed_item_ids`, and vice versa. That relationship cannot be enforced by
either field on its own.

Public fields would let any caller write `library.items[0].status =
LoanStatus::Available` and leave the member still holding the id — the library
now disagrees with itself, and nothing in the type system objects. Privacy
means `checkout` and `return_item` are the only code that can move an item
between states, so the invariant has exactly two places to be wrong instead of
unboundedly many.

The secondary benefit is freedom to change: swapping `Vec<Item>` for a
`HashMap<u32, Item>` to make lookups O(1) would break no caller, because no
caller can see the `Vec`.

### 10. What duplication does `late_fee_cents` remove?

The overdue arithmetic — "days held minus the loan period, clamped at zero,
times the daily rate". Without the default method, both `impl LoanTerms for
MediaKind` and `impl LoanTerms for Item` would carry their own copy, and the
day someone fixes an off-by-one in one of them the two silently disagree.
Instead the formula exists once, expressed in terms of the two required
methods, and every implementor inherits it.

Making it a free function would still deduplicate the arithmetic, but would
cost:

- **Method syntax.** `item.late_fee_cents(30)` becomes
  `late_fee_cents(item, 30)`, and discoverability goes with it — the method
  shows up on the type in documentation and autocomplete; the free function
  does not.
- **Membership in the contract.** As a trait method it is part of what
  `LoanTerms` *means*. A free function is a helper that happens to live nearby,
  and a new implementor of `LoanTerms` gets nothing automatically.
- **Overridability.** A future media kind with a fee cap could override the
  default body while keeping the trait's interface. A free function cannot be
  specialised per type.

### 11. Why is `Result` preferable to `panic!` for validation failures?

Because these failures are not bugs. A duplicate id or an already-borrowed book
is bad *data*, and correct code handed bad data should report the problem, not
abort. `Result` puts that outcome in the type signature, so the compiler makes
every caller acknowledge it, and pairing it with `Display` gives the user a
sentence they can act on rather than a stack trace. A panic, by contrast,
unwinds the thread and takes unrelated work down with it; a lending desk should
not lose its session because someone mistyped a member number.

A panic would be defensible where an *internal invariant* has broken, because
there is no sensible way to continue and no caller who could fix it. The
candidate in this crate is the member lookup in `return_item`: `borrower_id`
came out of a `LoanStatus` the library itself wrote, so if no such member
exists the library has already corrupted its own state. An
`.expect("borrower must exist; the library set this status")` there would be
honest — it documents the assumption and fails loudly if it is ever violated.

I returned `MemberNotFound` instead, on the grounds that the lookup stays in
the read phase and keeps the "validate everything before mutating anything"
property intact. The indexing in the write phase (`self.items[item_index]`) is
the same bet made implicitly — it panics on a broken invariant, and the index
came from the very vector being indexed.

### 12. Which derive did you deliberately leave off a type, and why?

`Clone` on `Item`. `Copy` is not even possible — `Item` owns two `String`s —
but `Clone` would derive fine, and it is left off on purpose.

The reason is that `Clone` is the escape hatch people reach for when the borrow
checker complains. With `#[derive(Clone)]` in place, a lookup returning
`Option<&Item>` that fights someone gets "fixed" by cloning, and now there is a
second copy of a catalogue entry that the library will never see updated. The
whole design rests on the library being the single owner of every item;
withholding `Clone` makes borrowing the only way to look at one, and the
compiler enforces it.

The contrast with `MediaKind` and `LoanStatus` is the point. Both derive `Copy`,
because they are small, own nothing on the heap, and duplicating one has no
aliasing consequence at all — which is exactly why `checkout` can lift a status
out of a vector with a plain read and stop borrowing. Copying a *description*
is harmless; copying an *identity* is not.

## Design notes

**Keeping the status and the borrowed list in agreement.** Three things do this
together. The fields are private, so `checkout` and `return_item` are the only
code that can move an item between states. Both are written as a read phase
followed by a write phase, so nothing is mutated until every check has passed —
a rejected checkout leaves the library byte-for-byte unchanged, rather than
half-updated. And within the write phase the two updates are adjacent and
unconditional: there is no path through either method that changes the item
without also changing the member.

**Indices instead of references.** The obvious way to write `checkout` — hold a
`&mut Item` and a `&mut Member` — is E0499. Rather than re-looking-up with
`iter_mut()` in the write phase and writing an `ok_or(...)?` that can never
fire, the read phase records two `usize` indices with `.position()`. A number
is not a borrow, so it crosses the phase boundary freely. The indices stay valid
because nothing is inserted or removed in between.

**Checked versus saturating arithmetic.** Both days are `u32` and both
subtractions can go negative, but they deserve different answers.
`late_fee_cents` uses `saturating_sub`, because returning inside the loan period
is a normal outcome that should clamp to a zero fee. `return_item` uses
`checked_sub`, because a return day earlier than the borrow day is a caller
error that should surface as `InvalidReturnDay`. Left unchecked, either would
panic in debug and wrap to a ten-figure fee in release.

**Ebooks are never late, by data rather than by branch.**
`MediaKind::Ebook`'s `daily_late_fee_cents` is `0`, so the shared fee formula
multiplies by zero and arrives at the right answer without a special case. No
future edit to `late_fee_cents` can forget the rule, because the rule is not
written there.

**Tie-breaking in `longest_loan_item`.** Two books share the 21-day maximum, and
`max_by_key` returns the *last* maximum, so the demo reports "Children of Dune"
rather than "Dune". The requirement — "the item that may be kept longest" — is
genuinely ambiguous when terms tie, so any of the tied items satisfies it. Noted
here because it looks like a bug and is not.

**The generic search (Part 9).** `filter_items<F: Fn(&Item) -> bool>` holds the
iterate-borrow-collect machinery once, and `items_by_author` and
`available_items` are now one-line calls into it. `Fn` rather than `FnMut` is
the weakest bound the callers need; a generic rather than `&dyn Fn` keeps the
predicate inlinable, since there is no need to store predicates anywhere.

The two named methods were kept rather than deleted in favour of the general
one. `available_items()` states something about lending;
`filter_items(|i| i.status == LoanStatus::Available)` makes every caller
re-derive it, and eventually one of them derives it wrong. The general method is
for questions the library did not anticipate; the named ones document the
questions it did.

## Example output

```text
== Catalogue ==
  #1 "Dune" by Frank Herbert (book, 320 pages) — available
  #2 "Children of Dune" by Frank Herbert (book, 180 pages) — available
  #3 "Project Hail Mary" by Andy Weir (audiobook, 540 minutes) — available
  #4 "The Rust Programming Language" by Steve Klabnik (ebook, 1200 kB) — available

Kept longest: Children of Dune — 21 days

== An on-time loan ==
  #3 "Project Hail Mary" by Andy Weir (audiobook, 540 minutes) — on loan to member 100 since day 0
  returned on day 10, owing $0.00

== A late return ==
  borrowed day 10, returned day 40, owing $2.25

== An ebook, very late ==
  held for a year, owing $0.00

== A handled error ==
  Item with ID 2 is already on loan by member 100

Ada is holding 1 of 3 items.
```
