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

### Parts 1–2 — Data model

`InputKind` is an enum because a Bitcoin transaction input is fundamentally one of
two mutually exclusive shapes: a **regular** input, which spends a previous
output and therefore carries an `OutPoint`, a `value`, and a `sequence`, or a
**coinbase** input, which mints new coins and carries a `block_height` and
`reward` instead of a previous output. These two shapes share no fields that
make sense to store together (a coinbase input has no `previous_output` to
point at, and a regular input has no `block_height`). Modelling them as one
struct with optional fields would let invalid states exist, such as a "regular"
input with no previous output, or a coinbase input that also references one.
An enum makes those states unrepresentable at compile time.

Because `InputKind` is an enum, any code that needs the input's value (or wants
to validate it) must `match` on it. The match is exhaustive — the compiler
rejects the code if a variant is left unhandled — so both `Regular` and
`Coinbase` cases are forced to be considered explicitly wherever an input is
consumed (see `BitcoinValue for InputKind` and `Transaction::validate`). This is
what lets `validate()` correctly reject a transaction that mixes coinbase and
regular inputs, or that contains more than one coinbase input: the match
arms make every combination visible instead of letting one case fall through
untested.

1. **What is a Bitcoin transaction input?** A reference to a previous
   transaction output that is being spent (or, for a coinbase transaction, an
   input that mints new block-subsidy coins instead of spending anything).
2. **What is a Bitcoin transaction output?** A destination for value: an amount
   in satoshis plus the conditions (recipient/output type) under which it can
   later be spent.
3. **What is a UTXO?** An "unspent transaction output" — an output that has been
   created by some past transaction but not yet consumed by any input. UTXOs
   are the spendable units of bitcoin.
4. **What does an outpoint identify?** A specific previous output: the `txid`
   of the transaction that created it and the `vout` index of that output
   within that transaction.
5. **How is a transaction fee calculated?** `fee = total_input_value -
   total_output_value`. It is the leftover value inputs provide that outputs
   do not claim, and it is paid to whoever mines/confirms the transaction.
6. **Why use integers rather than floating-point numbers for bitcoin amounts?**
   Floating-point arithmetic is not exact for base-10 fractional values and can
   accumulate rounding errors, which is unacceptable when moving real money.
   Satoshis (integer, indivisible units) avoid that entirely — every amount is
   a whole number of the smallest unit, so arithmetic is exact and
   reproducible.
7. **Why does `total_input_value()` borrow `self`?** It only needs to read the
   `inputs` field to sum values; it does not need to own or mutate the
   transaction. Borrowing (`&self`) lets the caller keep using the transaction
   afterwards and avoids an unnecessary clone.
8. **Why does `add_input()` take `&mut self`?** It mutates the transaction by
   pushing onto `self.inputs`, so it needs exclusive, mutable access to the
   transaction it's modifying.
9. **What happens when an input is moved into a transaction?** Ownership of the
   `InputKind` value transfers to the `Vec<InputKind>` inside the transaction.
   The caller's original variable becomes invalid to use afterwards (the
   compiler enforces this — see the ownership experiment below); the
   transaction is now solely responsible for that value's lifetime.
10. **Why is `Result` preferable to `panic!` for validation failures?**
    Validation failures (missing inputs, outputs exceeding inputs, etc.) are
    *expected*, recoverable conditions that arise from normal (if invalid)
    input data — not bugs in the program. `Result` lets the caller decide how
    to respond (reject the transaction, show an error, retry) without crashing
    the whole process. `panic!` is reserved for unrecoverable programmer
    errors/invariant violations, not for data the caller could reasonably
    supply.
11. **How do enums help model regular and coinbase inputs?** They express the
    "exactly one of these shapes" relationship directly in the type system, so
    every field a variant carries is always valid for that variant. Combined
    with exhaustive `match`, the compiler forces every part of the code that
    consumes an input to handle both cases explicitly.
12. **How does the `BitcoinValue` trait reduce duplication?** Both `TxOutput`
    and `InputKind` need "what is this worth in satoshis" and the derived
    "what is this worth in BTC" logic. `BitcoinValue` defines `value()` as the
    one thing each type must implement, and provides `value_in_btc()` once as
    a default method built on top of it — so the BTC-conversion formula is
    written a single time instead of once per type, and `total_input_value` /
    `total_output_value` can sum over either type generically via
    `BitcoinValue::value`.

### Part 7 — Ownership experiment

Attempting to use a value after moving it into `add_input`:

```rust
let coin = InputKind::Coinbase {
    block_height: 1,
    reward: 100,
};
transaction.add_input(coin);
println!("{coin}");
```

produces this compiler error:

```text
error[E0382]: borrow of moved value: `coin`
  --> src/main.rs:55:16
   |
50 |     let coin = InputKind::Coinbase {
   |         ---- move occurs because `coin` has type `InputKind`, which does not implement the `Copy` trait
...
54 |     transaction.add_input(coin);
   |                           ---- value moved here
55 |     println!("{coin}");
   |                ^^^^ value borrowed here after move

For more information about this error, try `rustc --explain E0382`.
error: could not compile `rfb-labs-week-2` (bin "rfb-labs-week-2") due to 1 previous error
```

**Explanation:** `add_input(&mut self, input: InputKind)` takes `InputKind` by
value, so calling `transaction.add_input(coin)` moves `coin` into the
function, and from there into `self.inputs`. `InputKind` does not derive
`Copy` (it owns a heap-allocated `String` inside its `OutPoint`, which isn't
`Copy`-able), so the move is a real transfer of ownership, not a bitwise copy.
After the move, `coin` in `main` is no longer a valid binding — the compiler
statically tracks that the value now lives inside the transaction and refuses
to let the old name be used, which is exactly the guarantee that prevents
use-after-move/double-free bugs without a garbage collector.

## Design notes

- **Validation order in `validate()`**: empty inputs/outputs are checked
  first (cheapest, most fundamental), then the zero-value-output rule, then
  the coinbase-mixing rules, then per-input txid validity, and finally the
  fee check (via `self.fee()?`) so `OutputsExceedInputs` is reported with
  accurate totals rather than being masked by an earlier, less specific error.
- **`select_utxos` trade-offs**: the required implementation is a simple
  first-fit walk over the slice in order, stopping as soon as the running
  total meets the target and returning `InsufficientFunds` (with the true
  total available) if the whole slice isn't enough. This is easy to reason
  about and deterministic, but it is not economical: it doesn't try to
  minimize the number of inputs (and therefore fees) or minimize leftover
  change.
  A better real-world algorithm (e.g. a variant of Bitcoin Core's branch-and-
  bound / knapsack-style selection) would prefer combinations of UTXOs whose
  total is as close as possible to the target with zero or minimal change,
  falling back to largest-first or smallest-first heuristics to control the
  number of inputs (and thus transaction weight/fees) and to avoid leaving
  UTXO "dust" behind. That is out of scope here since the assignment
  explicitly asks for slice-order selection, but it's the natural next step.
- **Part 10 (optional transaction-state extension) was not attempted** — out
  of scope for this submission.

## Example output

```
$ cargo run
transaction is valid
Transaction v2 (locktime 0)
  inputs: 2 (total 120000 sats)
  outputs: 2 (total 118000 sats)
  fee: 2000 sats
```
</content>
