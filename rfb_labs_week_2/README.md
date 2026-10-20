# Rust for Bitcoin 2.0 — Week 2

Build a simplified Bitcoin transaction model while practising structs, enums,
traits, ownership, borrowing, collections, and `Result`-based error handling.

The crate is intentionally incomplete. Search for `TODO` and implement each part;
do not change the public type names or function signatures.

## Recommended workflow

1. Read [ASSIGNMENT.md](ASSIGNMENT.md).
2. Complete Parts 3–5 in `transaction.rs` and `error.rs`.
3. Remove `#[ignore]` from the relevant test and run it.
4. Complete the traits and borrowing functions in Parts 6–7.
5. Build the payment example in `main.rs`.
6. Complete UTXO selection and its tests.
7. Add the remaining required tests yourself.

```bash
cargo test
cargo test -- --ignored
cargo run
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

`cargo test` checks the starter project. Ignored tests intentionally exercise
unfinished code; enable them progressively rather than leaving them ignored in the
submission.

## Written answers

Answer in your own words. Add the ownership compiler error from Part 7 as a fenced
text block, then explain what caused it.

1. What is a Bitcoin transaction input?

   An input points at a previous output and claims its value. A regular input
   references that output with an outpoint (`txid:vout`), says how many sats it
   unlocks, and carries a sequence number used for timelocks and replaceability.
   A coinbase input is special: it creates new sats out of thin air for the miner
   and is identified by block height instead of a previous output.

2. What is a Bitcoin transaction output?

   An output is a commitment of value: it says how many sats are being sent and
   to whom, described by a locking script (`output_type` here). Outputs become
   spendable UTXOs for the next transaction.

3. What is a UTXO?

   An unspent transaction output. It is one specific output from an earlier
   transaction that has not been spent yet and is therefore available to fund a
   new transaction. A wallet tracks its set of UTXOs as the money it can spend.

4. What does an outpoint identify?

   An outpoint is the unique reference to a specific output: the TXID of the
   transaction that created it plus the output index (`vout`). It is the "which
   coin" part of "spend this coin".

5. How is a transaction fee calculated?

   `fee = total_input_value - total_output_value`. Miners keep the difference,
   so the amount a sender actually pays is whatever they put in minus whatever
   they pay out to recipients and change.

6. Why use integers rather than floating-point numbers for bitcoin amounts?

   Floating point can't represent all values exactly (e.g. `0.1 + 0.2`), and
   rounding a monetary amount silently moves funds. Satoshis fit cleanly in
   `u64`; every operation stays exact, which is essential when dealing with
   money.

7. Why does `total_input_value()` borrow `self`?

   It only reads the inputs to sum their values. Taking `&self` lets the caller
   call it on an existing transaction without giving up ownership, and it can be
   called while other borrowed views of the transaction still exist.

8. Why does `add_input()` take `&mut self`?

   It mutates the transaction by pushing a new input onto its internal `Vec`.
   Mutating shared state requires exclusive access, which is what `&mut self`
   provides; it also communicates that the input being added is moved in.

9. What happens when an input is moved into a transaction?

   The transaction takes ownership of the `InputKind`, and the original variable
   is no longer usable. This prevents the same input being added twice or
   mutated after it is inside the transaction, which would corrupt the model.

10. Why is `Result` preferable to `panic!` for validation failures?

    Panicking crashes the whole program. Validation failures like
    `OutputsExceedInputs` or `InsufficientFunds` are expected, recoverable
    conditions; returning a `Result` lets the caller decide how to react (show
    an error, adjust inputs, abort gracefully) instead of dying.

11. How do enums help model regular and coinbase inputs?

    Both are inputs but have different data. An enum forces the compiler to
    handle every variant in any `match`, so a function that sums values or
    displays an input must explicitly decide what to do for both regular and
    coinbase cases — there is no way to accidentally forget one.

12. How does the `BitcoinValue` trait reduce duplication?

    Regular inputs store `value` and coinbase inputs store `reward`, and
    outputs store `value` too. `BitcoinValue` names that concept once, so
    `total_input_value()` and other code can call `.value()` on any of these
    instead of matching on the concrete types everywhere.

## Ownership compiler error

After calling `transaction.add_input(first)`, using `first` again triggers
error[E0382] because `add_input` moves it into the transaction:

```text
error[E0382]: borrow of moved value: `first`
  --> src/main.rs:24:5
   |
21 |     transaction.add_input(first);
   |                            ----- value moved here
22 |     // reusing `first` afterwards...
   |     ^^^^^^ value borrowed here after move
   |
   = note: move occurs because `first` has type `InputKind`, which does not
     implement the `Copy` trait
   = help: clone `first` if you want to keep using it, or restructure your code
     to not rely on the value after it has been moved
```

The compiler rejects this because `InputKind` is not `Copy`. Moving `first`
into `transaction` transfers ownership, so the binding `first` no longer owns
the data and any later use is a use-after-move. That is exactly the property we
want for the model: once a UTXO is added as an input, it is "gone" from the
sender's wallet.

## Design notes

- Validation order matters: structural checks (no inputs, no outputs) run first
  so later checks never read empty collections; coinbase rules come before the
  value checks.
- `validate()` reuses `fee()` so the outputs-exceed-inputs rule is defined once.
- `select_utxos` greedily takes UTXOs in slice order until the target is
  covered. This is simple and deterministic, but wasteful: it may take a large
  UTXO when a smaller one would suffice, and it ignores change minimization and
  privacy (avoiding address reuse). A better algorithm would sort candidates,
  pick the smallest UTXO that alone covers the target, and fall back to a
  largest-first combination only when needed.
- The optional transaction-state extension was not attempted in this submission.

## Example output

Paste the output of `cargo run` here once Part 8 is complete.

```text
Transaction v2 (locktime 0)
  inputs:  2 total 120000 sats
  outputs: 2 total 118000 sats
  fee:     2000 sats
```
