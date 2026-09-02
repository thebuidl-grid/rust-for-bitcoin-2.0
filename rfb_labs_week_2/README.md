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

## Written Answers

### Ownership experiment (Part 7)

I borrowed the highest-value output from a transaction and then tried to add
another output to the same transaction while that borrow was still alive:

```rust
let highest = highest_value_output(&transaction).unwrap();

// Attempt to mutate the transaction while `highest` still borrows from it.
transaction.add_output(TxOutput {
    value: 1_000,
    recipient: "bc1qextra".into(),
    output_type: OutputType::P2wpkh,
});

println!("{highest}");
```

Compiling this produced:

```text
error[E0502]: cannot borrow `transaction` as mutable because it is also borrowed as immutable
  --> examples/ownership_experiment.rs:15:5
   |
12 |     let highest = highest_value_output(&transaction).unwrap();
   |                                        ------------ immutable borrow occurs here
...
15 |     transaction.add_output(TxOutput { value: 1_000, recipient: "bc1qextra".into(), output_type: OutputType::P2wpkh });
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
16 |
17 |     println!("{highest}");
   |                ------- immutable borrow later used here
```

**What caused it:** `highest_value_output` returns `Option<&TxOutput>`, an
immutable reference borrowed from `transaction`. `add_output` takes `&mut
self`. Rust's borrow checker enforces that a value cannot be borrowed mutably
while an immutable borrow of it is still in use later (`highest` is used again
in the `println!`), because a `Vec::push` can reallocate the backing buffer
and invalidate any references into it — allowing both borrows at once would
let `highest` become a dangling pointer. The fix is to either drop/stop using
`highest` before mutating (e.g. print it immediately, or copy the `u64` value
out) or to finish all mutation before taking the borrow.

### Questions

1. **What is a Bitcoin transaction input?**
   A reference to a previous transaction output (an `OutPoint`) that is being
   spent, along with the data needed to authorize spending it (here modelled as
   `sequence`), or — for the first transaction in a block — a `Coinbase` input
   that creates new coins instead of spending an existing output.

2. **What is a Bitcoin transaction output?**
   A destination for value: an amount in satoshis (`value`), a recipient
   (`recipient`), and a locking condition (`output_type`) that says how the
   coins can later be spent (or, for `OpReturn`, that they carry data and are
   provably unspendable).

3. **What is a UTXO?**
   An **U**nspent **T**ransaction **O**utput — an output from some past
   transaction that has not yet been used as an input in a later transaction.
   UTXOs are the spendable "coins" a wallet tracks; `Utxo` in `utxo.rs` pairs an
   `OutPoint` with the value it holds.

4. **What does an outpoint identify?**
   A specific output of a specific transaction: the `txid` of the transaction
   that created the output, and the `vout` index of that output within the
   transaction. Together they uniquely identify one UTXO.

5. **How is a transaction fee calculated?**
   `fee = total_input_value - total_output_value`. Miners keep the difference,
   so it must never be negative; `Transaction::fee` uses `checked_sub` and
   returns `TransactionError::OutputsExceedInputs` instead of underflowing.

6. **Why use integers rather than floating-point numbers for bitcoin amounts?**
   Floating-point arithmetic is not exact — repeated addition/subtraction of
   BTC amounts accumulates rounding error, and two implementations can disagree
   on the "same" value. Satoshis are the smallest indivisible unit, so counting
   them as `u64` integers gives exact, deterministic arithmetic with no
   rounding, which is essential when money is involved.

7. **Why does `total_input_value()` borrow `self`?**
   It only needs to read the `inputs` field to sum values; it doesn't need to
   own or mutate the transaction. Taking `&self` lets callers keep using the
   transaction afterwards and lets many callers read it concurrently.

8. **Why does `add_input()` take `&mut self`?**
   It mutates the transaction by pushing onto the `inputs` `Vec`, which changes
   the transaction's state (and potentially reallocates its buffer). Mutation
   requires exclusive (`&mut`) access under Rust's borrowing rules.

9. **What happens when an input is moved into a transaction?**
   Ownership of the `InputKind` value transfers from the caller to the
   `Vec<InputKind>` inside the transaction. The caller's variable is no longer
   valid to use (unless the type were `Copy`, which `InputKind` is not because
   it owns a `String`); the transaction is now solely responsible for the
   input's memory and eventually for dropping it.

10. **Why is `Result` preferable to `panic!` for validation failures?**
    Invalid input data (empty inputs, outputs exceeding inputs, bad txids,
    etc.) is an expected, recoverable condition, not a programming bug.
    `panic!` unwinds/aborts the whole program and gives the caller no chance to
    react. `Result` makes the possibility of failure part of the function's
    type signature, forces callers to handle it via `?` or a `match`, and lets
    the caller decide what to do (reject the transaction, show an error,
    retry) instead of crashing.

11. **How do enums help model regular and coinbase inputs?**
    `InputKind` is an enum with `Regular { .. }` and `Coinbase { .. }`
    variants, each carrying only the fields that make sense for that kind of
    input (a coinbase input has no previous output to spend; a regular input
    does). Because it's an enum, the compiler forces every `match` to handle
    both variants — there is no way to accidentally treat a coinbase input as
    if it had a `previous_output`, and no invalid states are representable
    (unlike, say, an `Option<OutPoint>` field that a regular input forgot to
    set).

12. **How does the `BitcoinValue` trait reduce duplication?**
    `TxOutput` and `InputKind` each carry a monetary amount under a different
    field name (`value` vs. `value`/`reward` depending on variant). Implementing
    `BitcoinValue::value()` once per type lets `total_input_value` and
    `total_output_value` sum heterogeneous collections with the same
    `.iter().map(BitcoinValue::value).sum()` pattern, and gives every
    value-bearing type a shared `value_in_btc()` conversion for free via the
    trait's default method, without repeating the sats→BTC formula anywhere.

## Design notes

- **Validation order:** `validate()` checks structural problems (no inputs, no
  outputs) before input-shape problems (mixed/multiple coinbase, invalid
  txids) before value problems (zero-value outputs, outputs exceeding
  inputs), returning early with `?`/`return Err(..)` so the first violated
  rule is reported rather than the last.
- **Coinbase handling:** a transaction is either "all coinbase" (exactly one
  coinbase input, no regular inputs) or "all regular" — mixing the two is
  rejected, matching how real Bitcoin coinbase transactions work.
- **`Display` never panics on an invalid fee:** `Transaction`'s `Display`
  impl calls `self.fee()` and matches on the `Result`, printing
  `invalid (<reason>)` instead of unwrapping, so printing a transaction is
  always safe even for data that would fail `validate()`.
- **UTXO selection (Part 9):** `select_utxos` implements the simplest possible
  strategy — iterate the slice in order, greedily accumulating UTXOs until the
  running total meets or exceeds the target, then stop. It returns borrowed
  `&Utxo` references (no cloning) and reports `InsufficientFunds` with the
  slice's total value if the target can't be met.

  **Bonus — a better algorithm:** slice-order selection is simple but can be
  poor in practice: it ignores value entirely, so it might combine many small
  UTXOs when a single large one would do (bloating transaction size and fees),
  or leave awkward "dust" change. A better strategy for a real wallet would be
  a **branch-and-bound / largest-first with change-avoidance** approach (similar
  to Bitcoin Core's coin selection): prefer combinations that pay the target
  with zero or minimal change (avoiding an extra change output entirely when
  possible), otherwise prefer fewer, larger UTXOs to minimize transaction
  weight (and thus fee), and fall back to smallest-first for consolidating
  dust when the wallet has many small UTXOs sitting idle. Such an algorithm
  trades a little more computation (trying multiple candidate subsets) for a
  meaningfully smaller and cheaper resulting transaction.
- **Part 10 (optional transaction-state extension):** not attempted in this
  submission.

## Example output

```text
$ cargo run
transaction is valid
Transaction v2 (locktime 0): 2 input(s) totalling 120000 sats, 2 output(s) totalling 118000 sats, fee 2000 sats
```
