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
2. What is a Bitcoin transaction output?
3. What is a UTXO?
4. What does an outpoint identify?
5. How is a transaction fee calculated?
6. Why use integers rather than floating-point numbers for bitcoin amounts?
7. Why does `total_input_value()` borrow `self`?
8. Why does `add_input()` take `&mut self`?
9. What happens when an input is moved into a transaction?
10. Why is `Result` preferable to `panic!` for validation failures?
11. How do enums help model regular and coinbase inputs?
12. How does the `BitcoinValue` trait reduce duplication?

1. **What is a Bitcoin transaction input?**
   A reference to a previous transaction's unspent output that a transaction
   spends. In this model `InputKind::Regular { previous_output, value, sequence }`
   carries the `OutPoint` pointing at the prior output, how many sats it holds,
   and a `sequence`.

2. **What is a Bitcoin transaction output?**
   A destination a transaction pays to: an amount in sats, a recipient address,
   and a locking-script type. 

3. **What is a UTXO?**
   An *Unspent Transaction Output* — an output that has been created but not yet
   spent. It is a candidate input for a future transaction. `Utxo { outpoint,
   value }` models exactly that: where the coin sits (`OutPoint`) and how many
   sats it holds.

4. **What does an outpoint identify?**
   A single output of a single transaction, uniquely, via the pair `(txid,
   vout)` — `OutPoint { txid, vout }`, rendered as `<txid>:<vout>`. That
   uniqueness is what lets an input say precisely which UTXO it spends.

5. **How is a transaction fee calculated?**
   `total_inputs − total_outputs`: everything supplied minus everything sent out.
   In code, `fee()` computes `total_input_value().checked_sub(total_output_value())`,
   returning `OutputsExceedInputs` rather than underflowing when outputs exceed
   inputs.

6. **Why use integers rather than floating-point numbers for bitcoin amounts?**
   Floats cannot represent every value exactly (`0.1` is a repeating binary
   fraction), so repeated arithmetic drifts and a single wrong sat breaks a
   transaction. Integers are exact and sats are the smallest unit, so all math
   stays precise. 

7. **Why does `total_input_value()` borrow `self`?**
   It only reads the inputs to sum them; it must neither change nor consume the
   transaction. `&self` grants shared, read-only access, so it can be called many
   times while the transaction is still owned elsewhere. 

8. **Why does `add_input()` take `&mut self`?**
   It pushes onto `self.inputs`, mutating the transaction. Rust requires
   exclusive access (`&mut self`) for any mutation, which guarantees no other
   reference is live at the same time and rules out data races and aliasing bugs
   at compile time.

9. **What happens when an input is moved into a transaction?**
   Ownership transfers from the caller into the `Vec` stored in the transaction.
   After `add_input(input)` the caller can no longer use `input`; it now lives
   inside the transaction and is dropped with it. Each input must be built
   fresh per call.

10. **Why is `Result` preferable to `panic!` for validation failures?**
    A panic crashes the program; validation failures are expected, recoverable
    conditions (bad input, insufficient funds). Returning `Result<T,
    TransactionError>` lets the caller handle the failure, keeps the error type
    explicit, and makes failures directly testable with `assert_eq!`.

11. **How do enums help model regular and coinbase inputs?**
    `InputKind::Regular` and `InputKind::Coinbase` are mutually exclusive variants
    of one type. Matching (in `total_input_value`, `BitcoinValue`) forces the
    compiler to handle *every* variant, so coinbase inputs can never be silently
    ignored, and `validate()` can detect mixing the two kinds.

12. **How does the `BitcoinValue` trait reduce duplication?**
    Both `TxOutput` and `InputKind` produce a value in sats, stored under
    different field names (`value` vs `reward`). One trait declares `value()`;
    each type implements it in its own way, and shared logic such as
    `value_in_btc()` and `highest_value_output()` works uniformly on any
    implementor with no duplicated conversion code.

## Design notes

- **Coinbase vs regular inputs** are modelled as an enum rather than separate
  structs so the compiler forces every match arm to handle both, and mixing them
  is a detectable error in `validate()`.
- **Fee calculation** uses `checked_sub` and reports `OutputsExceedInputs` with
  both totals instead of panicking on an underflow.
- **UTXO selection** greedily takes UTXOs in slice order until the target is
  reached. Trade-off: simple and deterministic, but it can over-select (larger
  change than necessary). A smaller-fit or change-minimizing algorithm would
  produce tighter selections at the cost of more work; order-based selection was
  chosen to match the assignment's expected behaviour.
- **Part 10 state machine** was implemented with runtime-checked transitions: a
  `state` field on `Transaction` plus `mark_validated`, `sign`, `broadcast`,
  `confirm`, and `reject` methods that return `InvalidStateTransition` for
  illegal moves. A typestate (generics-on-type) design would move the checks to
  compile time but would complicate the existing `Transaction` API, so the
  runtime approach was preferred.

## Example output

`cargo run` (Part 8) prints the `Display` summary of the payment transaction
spending 70,000 and 50,000 sats, paying 90,000, returning 28,000 change, and
leaving a 2,000-sat fee:

```text
Transaction Summary
version: 2
locktime: 0
inputs:2
outputs:2
total_output_value: 118000
total_input_value: 120000
```

## Compiler Output
```bash

Compiling rfb-labs-week-2 v0.1.0 (/home/test/Desktop/rust/rust-for-bitcoin-2.0/rfb_labs_week_2)
error[E0599]: the method `to_owned` exists for struct `TxOutput`, but its trait bounds were not satisfied
--> src/transaction.rs:209:17
|
14 | pub struct TxOutput {
| ------------------- method `to_owned` not found for this struct because it doesn't satisfy `TxOutput: Clone` or `TxOutput: ToOwned`
...
209 |     Some(output.to_owned())
|                 ^^^^^^^^ method cannot be called on `TxOutput` due to unsatisfied trait bounds
|
= note: the following trait bounds were not satisfied:
`TxOutput: Clone`
which is required by `TxOutput: ToOwned`
help: consider annotating `TxOutput` with `#[derive(Clone)]`
|
14 + #[derive(Clone)]
15 | pub struct TxOutput {
|

error[E0308]: mismatched types
--> src/transaction.rs:219:5
|
215 |   ) -> Vec<&'a TxOutput> {
|        ----------------- expected `Vec<&'a TxOutput>` because of return type
...
219 | /     *transaction.outputs
220 | |         .iter()
221 | |         .filter(|output| output.recipient == recipient)
222 | |         .collect::<Vec<&TxOutput>>()
| |____________________________________^ expected `Vec<&TxOutput>`, found `[&TxOutput]`
|
= note: expected struct `Vec<&'a TxOutput>`
found slice `[&TxOutput]`
help: try using a conversion method
|
219 ~     (*transaction.outputs
220 |         .iter()
221 |         .filter(|output| output.recipient == recipient)
222 ~         .collect::<Vec<&TxOutput>>()).to_vec()
|

warning: unused variable: `previous_output`
--> src/transaction.rs:84:42
|
84 |                     InputKind::Regular { previous_output, value, .. } => {
|                                          ^^^^^^^^^^^^^^^ help: try ignoring the field: `previous_output: _`
|
= note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default
```

**What caused the Part 7 errors:**

- `error[E0599]` — I tried to return an *owned* copy of the highest-value output
  with `.to_owned()`. The borrow checker refused because `TxOutput` does not
  implement `Clone` (or `ToOwned`), and even if it did, the exercise is to return
  a *borrowed* reference. The fix is returning `&TxOutput` borrowed from the
  transaction instead of copying.
- `error[E0308]` — `*transaction.outputs` dereferences the borrowed field and
  `.collect()` yields `[&TxOutput]` while the signature demands `Vec<&'a
  TxOutput>`; the mismatch is about the exact target collection type and
  lifetime. The fix is collecting directly into `Vec<&TxOutput>` bound to the
  transaction's lifetime `'a`, without cloning.

