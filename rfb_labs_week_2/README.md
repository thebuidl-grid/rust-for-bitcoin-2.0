# Rust for Bitcoin 2.0 — Week 2

Build a simplified Bitcoin transaction model while practising structs, enums,
traits, ownership, borrowing, collections, and `Result`-based error handling.

All monetary values are integer satoshis (`1 BTC = 100,000,000 sats`). The model
does not serialize, sign, or broadcast a real transaction.

## Running it

```bash
cargo test
cargo run
cargo run --example ownership_experiment
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

Current status: **36 tests passing, none ignored** (32 integration tests plus 4 doc
tests, three of which are `compile_fail` checks on the Part 10 state machine);
`cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings`
both clean.

## Written answers

### The ownership error from Part 7

The experiment lives in [`examples/ownership_experiment.rs`](examples/ownership_experiment.rs).
Uncommenting the marked line produces:

```text
error[E0382]: borrow of moved value: `input`
  --> examples/ownership_experiment.rs:18:20
   |
 6 |     let input = InputKind::Regular {
   |         ----- move occurs because `input` has type `InputKind`, which does not implement the `Copy` trait
...
15 |     transaction.add_input(input);
   |                           ----- value moved here
...
18 |     println!("{}", input.value());
   |                    ^^^^^ value borrowed here after move

For more information about this error, try `rustc --explain E0382`.
error: could not compile `rfb-labs-week-2` (example "ownership_experiment") due to 1 previous error
```

**What caused it.** `add_input` takes its parameter by value, so calling
`transaction.add_input(input)` *moves* the input into the transaction. The local
variable `input` is dead from that line onwards, and the later `input.value()` tries
to borrow something that no longer belongs to this scope.

The error message names the reason precisely: `InputKind` does not implement `Copy`.
It cannot, because it owns a `String` (the txid inside its `OutPoint`), and a `String`
owns a heap allocation. Copying it bitwise would leave two owners of one allocation
and a double free when both went out of scope. So Rust moves instead of copying, and
then refuses to let the old name be used.

This is desirable here rather than annoying. A transaction input represents a specific
coin being spent. Having two independently usable copies of one input would model
something that cannot exist — the same coin being spent twice. The compiler is
enforcing a Bitcoin rule for free.

Reading the value back through the transaction compiles fine, because that borrows
from the new owner instead of the old name.

### Questions

**1. What is a Bitcoin transaction input?**

An input is a reference to a coin being spent, plus the proof that the spender is
allowed to spend it. It does not contain money itself — it points at an existing
unspent output somewhere earlier in the chain and consumes it. In this model that is
the `InputKind::Regular` variant, holding a `previous_output` pointing at the coin, a
`value`, and a `sequence` number.

**2. What is a Bitcoin transaction output?**

An output is a chunk of value plus the condition for spending it later. Here that is
`TxOutput`: a `value` in satoshis, a `recipient`, and an `output_type` describing the
kind of lock. Once a transaction is confirmed, its outputs are the new spendable coins
and they sit there untouched until someone references them as an input.

**3. What is a UTXO?**

An Unspent Transaction Output — an output that no later transaction has consumed yet.
The set of all UTXOs is the complete record of who owns what. There is no balance
stored anywhere in Bitcoin; a wallet balance is calculated by finding every UTXO its
keys can unlock and adding them up. That is why `select_utxos` works over a collection
of discrete chunks rather than a single number.

**4. What does an outpoint identify?**

Exactly one output of exactly one past transaction. It is a pair — a transaction id
and an index — because a transaction usually has several outputs, so the txid alone is
ambiguous. `OutPoint { txid, vout }` displays as `<txid>:<vout>`, which is how it is
written everywhere in Bitcoin tooling. It is the coin's unique address within the UTXO
set.

**5. How is a transaction fee calculated?**

Total inputs minus total outputs. Nothing else. There is no fee field and no fee
output — the fee is simply the value that goes in and does not come out, and the miner
who includes the transaction claims the difference.

That is why `fee()` returns a `Result`. Doing `inputs - outputs` on a `u64` where the
outputs are larger would wrap around to an astronomical number instead of going
negative, reporting a colossal fee for a transaction that is merely invalid.
`checked_sub` catches it and returns `OutputsExceedInputs` carrying both totals.

**6. Why use integers rather than floating-point numbers for bitcoin amounts?**

Because `f64` cannot represent most decimal fractions exactly. `0.1 + 0.2` is not
`0.3` in binary floating point, and those tiny errors accumulate across arithmetic.
For money that is unacceptable: a rounding error is either lost coins or invented
ones, and consensus requires every node to compute byte-identical results.

Satoshis are indivisible, so `u64` represents every possible amount exactly, and
addition and subtraction are exact. `u64` also comfortably holds the 2.1 quadrillion
satoshis that will ever exist. The one place a float appears is `value_in_btc()`, which
is for human display only and never feeds back into arithmetic.

**7. Why does `total_input_value()` borrow `self`?**

Because it only reads. Taking `self` by value would consume the transaction just to
ask its total, so the caller could never use it again — and `Display` calls it while
formatting, which would be impossible. Taking `&mut self` would be dishonest, claiming
a right to modify that the method does not need, and would stop two reads happening at
once. `&self` says exactly what the method does: look, don't touch.

**8. Why does `add_input()` take `&mut self`?**

Because pushing onto `self.inputs` mutates the transaction. A shared `&self` reference
cannot modify what it points at, so an immutable borrow would not compile. `&mut self`
is an exclusive borrow: while it is held, nothing else can read or write the
transaction, which is what makes the mutation safe without a lock.

**9. What happens when an input is moved into a transaction?**

Ownership transfers to the transaction. The value is not copied — the transaction takes
over responsibility for it, including freeing the heap memory behind its `String` when
the transaction is dropped. The original variable becomes unusable, and touching it
afterwards is the `E0382` error above.

Practically: the input now lives inside `transaction.inputs`, and the only way to reach
it is by borrowing from the transaction.

**10. Why is `Result` preferable to `panic!` for validation failures?**

Because invalid input is expected, not exceptional. A transaction with more outputs
than inputs is an ordinary thing for a validator to encounter, and the correct response
is to reject it and carry on — not to abort the process. A node that panicked on every
malformed transaction it received would be trivially easy to shut down.

`Result` also puts the failure in the type signature, so callers cannot forget to
handle it, and the error can carry useful context: `OutputsExceedInputs` reports both
totals rather than just saying "invalid". And it composes — `validate()` uses `?` to
propagate the error from `fee()` instead of re-deriving it. `panic!` belongs to bugs in
the program, not bad data arriving at it.

**11. How do enums help model regular and coinbase inputs?**

The two kinds of input carry genuinely different data. A regular input references a
previous output and has a sequence number; a coinbase input creates new coins and has a
block height instead — it has no previous output at all, because there isn't one.

A single struct would need optional fields for both shapes, and then every reader would
have to remember which combinations are legal and handle nonsense like a coinbase with a
previous output. The enum makes those states unrepresentable: if you have a `Coinbase`,
there is no `previous_output` field to misuse.

Matching then forces both cases to be handled. `total_input_value` must say what a
coinbase contributes, because the compiler rejects a non-exhaustive match. Adding a
third variant later would produce errors at exactly the places needing updating, rather
than silently wrong results.

**12. How does the `BitcoinValue` trait reduce duplication?**

It gives one name — `value()` — to a concept that is spelled differently on each type.
`TxOutput` stores `value`, `InputKind::Regular` stores `value`, and
`InputKind::Coinbase` stores `reward`. Without the trait, code that just wants "how much
is this worth" needs to know which type it is holding and which field name applies.

The bigger saving is the default method. `value_in_btc()` is written once in the trait
and every implementor gets it free — the satoshi-to-BTC conversion exists in exactly one
place, so it cannot drift or be miscopied. Implementors only supply `value()`, the one
piece that genuinely differs per type.

## Design notes

**Validation order.** `validate()` checks structure before arithmetic: missing inputs,
missing outputs, coinbase rules, txid sanity, zero-value outputs, and only then the fee.
The reasoning is that a structural problem is more fundamental than a value problem — a
transaction with no inputs shouldn't be reported as "outputs exceed inputs" merely
because zero is less than something. The final fee check reuses `fee()` via `?` instead
of repeating the comparison, so the rule exists in one place.

**Zero-value outputs.** Rejected unless the output type is `OpReturn`. Real Bitcoin uses
zero-value `OP_RETURN` outputs to embed data, which is a legitimate reason to create an
output worth nothing. Any other zero-value output is provably unspendable.

**Borrowing over cloning.** `highest_value_output`, `find_outputs_for_recipient`, and
`select_utxos` all return references into data the caller owns. Nothing is copied, and
the lifetimes stop a result outliving its source. `find_outputs_for_recipient` uses an
explicit `'a` to tie the returned references to the *transaction* rather than the
recipient string, so the recipient can be a short-lived temporary.

Two tests assert this with `std::ptr::eq`, which fails if the implementation ever
starts cloning.

**UTXO selection trade-offs.** The implemented algorithm is first-fit in slice order:
walk the list, accumulate until the target is covered, stop. It is simple, predictable,
and easy to test. It is also a poor strategy in practice, for three reasons:

- *It ignores size.* Paying 1,000 sats from a wallet whose first UTXO is 10 BTC spends
  the large coin and creates a large change output, when a small one would have done.
- *It creates dust.* Selecting without regard to the target produces change outputs that
  can cost more to spend later than they are worth.
- *It leaks information.* Deterministic ordering makes a wallet's behaviour easy to
  fingerprint on a public chain.

Better approaches, roughly in order of effort:

- **Branch and bound**, which Bitcoin Core uses. It searches for a combination whose
  total matches the target plus fee closely enough to need *no change output at all*.
  That is the ideal result: a smaller transaction, a lower fee, no dust, and one less
  output for chain analysis to link.
- **Knapsack / random draw** as a fallback when no changeless match exists, minimising
  waste rather than taking the first coins that fit.
- **Fee-aware selection**, accounting for the fact that each extra input makes the
  transaction physically larger and therefore more expensive. Sometimes one large input
  is cheaper than three small ones even though it creates more change.

I kept first-fit because the assignment specifies input order and the tests pin that
behaviour, but the interesting engineering is in what it gets wrong.

**Part 10 — transaction states.** Attempted, in [`src/state.rs`](src/state.rs), using
the typestate pattern:

```text
Created ──validate()──> Validated ──sign()──> Signed ──broadcast()──> Broadcast
   │                                                                      │
   └──────────────── Rejected <────────── reject() ───────────────────────┤
                                                                          │
                                             Confirmed <───confirm()──────┘
```

`Lifecycle<S>` carries the state in the *type parameter* rather than in a field, and
each state is its own struct. Every transition takes `self` by value and returns a
`Lifecycle` of a different type.

Two properties fall out of that, both for free:

- **Invalid transitions do not exist.** There is no `broadcast()` on
  `Lifecycle<Created>` to call, so skipping validation is not a runtime error to be
  caught — it is a program that cannot be written. No `if state == ...` guard is
  needed anywhere, and no test can reach the bad path because the bad path does not
  compile.
- **Nothing can be done twice.** `broadcast()` consumes the `Signed` value, so a
  second call has no value left to operate on. Double-broadcast is the same class of
  error as the Part 7 use-after-move, and the compiler rejects it for the same reason.

I verified both claims rather than asserting them: `src/state.rs` carries three
`compile_fail` doc tests — broadcasting from `Created`, signing before validating, and
broadcasting twice. `cargo test` fails if any of them ever starts compiling, so the
guarantee is regression-tested rather than merely intended.

States carry data where it is meaningful: `Signed` holds the signature, `Confirmed`
holds the block height, and `Rejected` holds a `RejectionReason` distinguishing local
validation failure from network refusal. `Rejected` is reachable from two places, which
matches reality — a transaction can be refused before it is ever sent, or accepted
locally and then dropped by the network.

The transaction stays readable in every state through `transaction()`, including after
rejection, because a rejected transaction is still worth inspecting.

The trade-off worth naming: typestate is awkward when the state must be chosen at
runtime — you cannot put `Lifecycle<Created>` and `Lifecycle<Signed>` in the same `Vec`,
since they are different types. A wallet tracking many transactions at mixed stages
would need an ordinary `enum` wrapper around them, giving up compile-time transition
checking for the ability to store them together. This model tracks one transaction at a
time, so the compile-time guarantee is worth more.

## Example output

```text
$ cargo run
Selected 2 UTXO(s) to cover 92000 sats:
  1111111111111111111111111111111111111111111111111111111111111111:0 worth 70000 sats
  2222222222222222222222222222222222222222222222222222222222222222:1 worth 50000 sats

Validation: passed
Transaction (version 2, locktime 0)
  inputs:  2 (120000 sats)
  outputs: 2 (118000 sats)
  fee:     2000 sats

  input:  regular input spending 1111111111111111111111111111111111111111111111111111111111111111:0 worth 70000 sats (sequence 4294967295)
  input:  regular input spending 2222222222222222222222222222222222222222222222222222222222222222:1 worth 50000 sats (sequence 4294967295)
  output: 90000 sats to bc1qreceiver (P2WPKH)
  output: 28000 sats to bc1qsender (P2WPKH)

Largest output: 90000 sats to bc1qreceiver (P2WPKH) (0.00090000 BTC)
```

The change output is derived rather than hardcoded: 120,000 sats in, minus the 90,000
payment, minus the intended 2,000 fee, leaves 28,000 returning to the sender.
