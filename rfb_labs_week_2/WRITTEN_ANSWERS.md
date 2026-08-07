# Week 2 — Written Answers

Companion to [README.md](README.md) / [ASSIGNMENT.md](ASSIGNMENT.md): the written
answers, ownership experiment, design notes, and example output requested by the
"Written answers" / "Design notes" / "Example output" sections.

## Written answers

1. **What is a Bitcoin transaction input?**
   A reference to a previous transaction output that is being spent. In this model
   an input is either `InputKind::Regular`, which points at an existing UTXO via an
   `OutPoint` and carries its value and sequence number, or `InputKind::Coinbase`,
   which mints new value at a given block height instead of spending a prior output.

2. **What is a Bitcoin transaction output?**
   A new destination for value created by the transaction: an amount in satoshis,
   a recipient, and an output type (`P2pkh`, `P2wpkh`, `P2tr`, or `OpReturn` for
   non-spendable data). Outputs become the UTXOs that future transactions can spend.

3. **What is a UTXO?**
   An "unspent transaction output" — a `TxOutput` from a confirmed transaction that
   has not yet been consumed as an input elsewhere. `Utxo` in `utxo.rs` pairs the
   `OutPoint` that identifies it with its value so it can be selected for spending.

4. **What does an outpoint identify?**
   A specific output of a specific transaction: the transaction's `txid` plus the
   `vout` index of the output within that transaction. Together they uniquely
   identify one spendable (or already-spent) output on the chain.

5. **How is a transaction fee calculated?**
   `fee = total_input_value - total_output_value`. `Transaction::fee` sums all input
   values and all output values, then uses `checked_sub` so that if outputs would
   exceed inputs it returns `TransactionError::OutputsExceedInputs` instead of
   underflowing an unsigned integer.

6. **Why use integers rather than floating-point numbers for bitcoin amounts?**
   Floating-point arithmetic is not exact for decimal fractions, so repeated
   addition/subtraction of BTC amounts as `f64` accumulates rounding error and can
   make two logically-equal amounts compare unequal. Satoshis are the smallest
   indivisible unit, so representing value as `u64` satoshis makes every amount
   exact and lets validation use exact, checked integer arithmetic.

7. **Why does `total_input_value()` borrow `self`?**
   It only needs to read the `inputs` vector to sum values — it does not need to
   mutate or take ownership of the transaction. Borrowing (`&self`) lets the caller
   keep using the transaction afterward and lets multiple read-only calls
   (`total_input_value`, `total_output_value`, `fee`, `Display`) coexist.

8. **Why does `add_input()` take `&mut self`?**
   Pushing a new `InputKind` onto `self.inputs` mutates the transaction's internal
   `Vec`, so the method needs exclusive, mutable access to `self`. An immutable
   borrow would not compile against `Vec::push`.

9. **What happens when an input is moved into a transaction?**
   `add_input(&mut self, input: InputKind)` takes `input` by value, so ownership of
   the `InputKind` transfers from the caller into `self.inputs` when it is pushed.
   The caller's original binding is no longer usable — see the ownership experiment
   below, which demonstrates the identical rule for `add_output`.

10. **Why is `Result` preferable to `panic!` for validation failures?**
    Invalid transaction data (empty inputs, mixed coinbase inputs, outputs
    exceeding inputs, etc.) is an *expected*, recoverable condition, not a bug in
    the program. `Result<(), TransactionError>` lets callers inspect *why*
    validation failed and decide how to respond (reject the transaction, show an
    error, retry), whereas `panic!` would unwind/abort the whole program and is
    only appropriate for truly unrecoverable programmer errors.

11. **How do enums help model regular and coinbase inputs?**
    `InputKind` is an enum with `Regular { previous_output, value, sequence }` and
    `Coinbase { block_height, reward }` variants because the two kinds of input are
    mutually exclusive and carry different data — a coinbase input has no previous
    output to spend. Because it's an enum, every `match` on an `InputKind` is
    exhaustive: the compiler forces both variants to be handled (e.g. in
    `BitcoinValue::value`, `Display`, and the coinbase-counting logic in
    `validate`), so it's impossible to accidentally forget the coinbase case.

12. **How does the `BitcoinValue` trait reduce duplication?**
    `TxOutput` and `InputKind` both need a "value in satoshis" and a "value in BTC"
    concept, but they store that value differently (`TxOutput::value` vs.
    `InputKind`'s per-variant `value`/`reward` fields). `BitcoinValue` defines a
    single required method, `value(&self) -> u64`, that each type implements
    according to its own layout, plus a shared default method,
    `value_in_btc(&self) -> f64`, implemented once against `value()`. Code that
    sums totals (`total_input_value`, `total_output_value`) can then call
    `BitcoinValue::value` generically over `iter().map(...)` without caring which
    concrete type it's summing, and BTC conversion never needs to be reimplemented.

## Ownership experiment (Part 7)

To confirm that `add_output` takes ownership of its argument, I built a small
example that constructs a `TxOutput`, moves it into a transaction with
`add_output`, and then tries to use the original binding again:

```rust
let mut transaction = Transaction::new(2, 0);
let output = TxOutput {
    value: 1_000,
    recipient: "bc1qreceiver".into(),
    output_type: OutputType::P2wpkh,
};

transaction.add_output(output);

// Using `output` here after it has been moved does not compile.
println!("{output}");
```

Compiling it produces:

```text
error[E0382]: borrow of moved value: `output`
  --> examples/ownership_experiment.rs:18:16
   |
 9 |     let output = TxOutput {
   |         ------ move occurs because `output` has type `TxOutput`, which does not implement the `Copy` trait
...
15 |     transaction.add_output(output);
   |                            ------ value moved here
...
18 |     println!("{output}");
   |                ^^^^^^ value borrowed here after move

For more information about this error, try `rustc --explain E0382`.
```

**Explanation:** `add_output(&mut self, output: TxOutput)` takes `output` by value,
not by reference, so calling it moves the `TxOutput` out of the caller's `output`
binding and into `self.outputs`. `TxOutput` holds a heap-allocated `String`
(`recipient`) and does not derive `Copy`, so the compiler cannot implicitly
duplicate it — after the move, the original `output` binding is no longer valid,
and any later use is a compile-time error rather than a runtime bug. This is the
same rule that lets `Transaction` own its inputs/outputs outright while
`highest_value_output` and `find_outputs_for_recipient` instead borrow (`&`) so
they can inspect the data without taking ownership away from the transaction.

## Design notes

- **Error modelling:** `TransactionError` is a single enum covering every expected
  validation and selection failure, each with a `Display` impl that explains what
  went wrong (and, where useful, the concrete numbers involved — e.g.
  `OutputsExceedInputs { total_inputs, total_outputs }` and
  `InsufficientFunds { available, required }`). `validate()` returns as soon as it
  finds the first violated rule, checking structural issues (no inputs/outputs,
  zero-value non-`OP_RETURN` outputs) before input-composition rules (coinbase
  mixing/duplication, empty txids), and finally reuses `fee()` via `?` to catch
  outputs exceeding inputs without duplicating the checked-subtraction logic.
- **Borrowing over cloning:** `highest_value_output` and `find_outputs_for_recipient`
  both return borrowed `&TxOutput`/`Vec<&TxOutput>` tied to the transaction's
  lifetime rather than cloning outputs, so read-only queries never allocate new
  `TxOutput`s or `String`s just to inspect existing data.
- **UTXO selection trade-offs (Part 9):** `select_utxos` implements the simplest
  possible strategy — iterate the slice in the order given, accumulate until the
  target is met, and borrow only the UTXOs actually used. This is easy to reason
  about and cheap (`O(n)`, no sorting, no cloning), but it is not necessarily
  *good* selection: it ignores value entirely, so it can select many small UTXOs
  when one large one would do (bloating transaction size/fees) or leave an
  awkward amount of change. A better real-world algorithm would:
  - Prefer a **single UTXO ≥ target** if one exists (minimizes inputs and fee).
  - Otherwise fall back to **largest-first** or **branch-and-bound** (as Bitcoin
    Core's wallet does) to minimize the number of inputs and the resulting change
    output, which reduces both fees and the growth of the UTXO set.
  - Optionally randomize among equally good candidates to avoid leaking spending
    patterns tied to UTXO age/order (a privacy consideration, not just an
    efficiency one).
  The in-order/first-fit approach here was kept deliberately simple to match the
  scope of the assignment (borrowing over a slice, returning `InsufficientFunds`),
  with the trade-offs documented rather than implemented.
- **Optional state machine (Part 10):** not attempted — out of scope for this
  submission; the model here stops at validating a fully-formed `Transaction`.

## Example output

```text
$ cargo run
transaction is valid

Transaction v2 (locktime 0)
  inputs:  2
  outputs: 2
  total input:  120000 sats
  total output: 118000 sats
  fee: 2000 sats
```
