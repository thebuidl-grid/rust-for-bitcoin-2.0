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

1. **What is a Bitcoin transaction input?**
   An input is a reference to a previous transaction output being spent. It proves
   ownership of that output's value and consumes it entirely — you can't spend
   part of an input; the full value is consumed, and anything not sent to an
   output becomes the miner fee. In this model, `InputKind::Regular` carries the
   `OutPoint` (which previous output it spends) plus the value and sequence
   number; `InputKind::Coinbase` is the special case where a miner creates new
   value directly rather than spending an existing output.

2. **What is a Bitcoin transaction output?**
   An output is a destination for value — an amount plus a locking condition
   (here modelled as `recipient` + `output_type`) that some future input must
   satisfy to spend it. A transaction's outputs become the next transaction's
   available inputs (UTXOs), which is exactly what `Utxo` represents in
   `utxo.rs`.

3. **What is a UTXO?**
   An Unspent Transaction Output — an output that hasn't been consumed by any
   later input yet. It's the actual "coin" a wallet holds; a wallet's balance is
   the sum of all UTXOs it can spend, not a single running number.

4. **What does an outpoint identify?**
   `OutPoint { txid, vout }` identifies one specific output uniquely — the
   transaction that created it (`txid`) plus that output's position within that
   transaction's output list (`vout`), since a single transaction can have many
   outputs.

5. **How is a transaction fee calculated?**
   `fee = total_input_value − total_output_value`. There's no separate "fee
   field" in a transaction — it's whatever value goes in but isn't accounted for
   by any output, and it's implicitly claimed by whoever mines the transaction
   into a block. `Transaction::fee()` here uses `checked_sub` specifically so
   that if outputs somehow exceed inputs (which should be impossible in a valid
   transaction), it returns `Err(OutputsExceedInputs { .. })` instead of
   underflowing/panicking.

6. **Why use integers rather than floating-point numbers for bitcoin amounts?**
   Floating-point numbers can't represent most decimal fractions exactly in
   binary, so repeated arithmetic on `f64` BTC amounts accumulates tiny rounding
   errors (I hit this for real in Week 1's `calculate_fee`, where
   `0.4 + 0.4 + 0.4 - 1.0` came out as `0.00000999999999996` instead of a clean
   `0.00001`). Using `u64` satoshis avoids the problem entirely — satoshis are
   the smallest unit, so every real amount is a whole number, and integer
   subtraction/addition is always exact.

7. **Why does `total_input_value()` borrow `self`?**
   It only needs to read the transaction's inputs to sum them — it doesn't need
   to own or modify the `Transaction`. Borrowing (`&self`) lets the caller keep
   using the transaction afterward, and lets multiple read-only calls (like
   `total_input_value()` and `total_output_value()` inside `fee()`) happen
   without any of them needing to fight over ownership.

8. **Why does `add_input()` take `&mut self`?**
   Because it actually changes the transaction — it pushes a new `InputKind`
   onto `self.inputs`. Any method that mutates a field needs `&mut self`
   specifically so the compiler can guarantee no other borrow of the transaction
   exists at the same time it's being changed.

9. **What happens when an input is moved into a transaction?**
   Ownership of the `InputKind` value transfers to the `Vec<InputKind>` inside
   the transaction — the variable that used to own it (e.g. a local variable
   holding the input before calling `add_input`) is no longer valid to use
   afterward. I proved this directly for Part 7: constructing an output, calling
   `add_output(output)`, then trying to read `output.value` afterward fails to
   compile with `E0382: borrow of moved value` (full error below), because the
   value's ownership already moved into the transaction's `Vec` on the previous
   line.

10. **Why is `Result` preferable to `panic!` for validation failures?**
    Validation failures (empty inputs, outputs exceeding inputs, mixed coinbase
    inputs, etc.) are *expected*, recoverable conditions — a caller should be
    able to check whether a transaction is valid and react accordingly (reject
    it, show an error to a user), not have the whole program crash. `panic!` is
    for programmer bugs / truly unrecoverable states, not for "the input data
    didn't pass a rule." Returning `Result<(), TransactionError>` lets the
    caller decide what to do, and the compiler forces every caller to at least
    acknowledge the possibility of failure.

11. **How do enums help model regular and coinbase inputs?**
    `InputKind` is an enum with two variants, `Regular` and `Coinbase`, each
    carrying different data (a regular input has a `previous_output` + `value` +
    `sequence`; a coinbase input has a `block_height` + `reward`, no previous
    output at all, since it creates value rather than spending it). An enum
    makes it *impossible* to represent an invalid mixture (like a coinbase input
    that also has a previous output) — the data only exists in the shape that's
    valid for that variant. And because it's an enum, every `match` on an
    `InputKind` (in `total_input_value`, `value()`, `Display`, `validate`) is
    checked by the compiler for exhaustiveness — if a third variant were ever
    added, every one of those `match` blocks would fail to compile until updated
    to handle it, so it's impossible to silently forget a case.

12. **How does the `BitcoinValue` trait reduce duplication?**
    `TxOutput` and `InputKind` are different types with value stored under
    different field names (`value` vs. `value`/`reward` depending on variant),
    but both conceptually "have a value in satoshis." `BitcoinValue` gives them
    one shared interface (`.value()`), plus a default method (`value_in_btc()`)
    that's implemented *once*, in terms of `.value()`, and automatically works
    for both types without either needing to repeat the sats-to-BTC conversion
    logic themselves.

### Part 7 — ownership experiment

Deliberately triggered by adding this to `main.rs` temporarily:

```rust
let extra_output = TxOutput {
    value: 1_000,
    recipient: "bc1qtest".to_string(),
    output_type: OutputType::P2wpkh,
};
transaction.add_output(extra_output);
println!("{}", extra_output.value);
```

The resulting compiler error:

```
error[E0382]: borrow of moved value: `extra_output`
  --> src/main.rs:49:20
   |
43 |     let extra_output = TxOutput {
   |         ------------ move occurs because `extra_output` has type `TxOutput`, which does not implement the `Copy` trait
...
48 |     transaction.add_output(extra_output);
   |                            ------------ value moved here
49 |     println!("{}", extra_output.value);
   |                    ^^^^^^^^^^^^^^^^^^ value borrowed here after move
```

**What caused it:** `TxOutput` doesn't implement `Copy` (it owns a heap-allocated
`String` in `recipient`, and `Copy` types must be safely duplicable with a plain
bitwise copy — a `String` can't be, since two independent owners of the same heap
allocation would both try to free it). Because it's not `Copy`, passing
`extra_output` by value into `add_output(&mut self, output: TxOutput)` **moves**
ownership of it into the transaction's `outputs` vector. After that move, the
original `extra_output` variable is no longer valid — the compiler tracks this
statically and refuses to compile any later use of it, which is exactly what
`println!("{}", extra_output.value)` on the next line tried to do. This is Rust
preventing a real bug at compile time: without this check, you could accidentally
read/use a value that logically no longer belongs to you (it now belongs to the
transaction), which could easily hide a mistake like double-counting an output's
value elsewhere.

## Design notes

- **UTXO selection (Part 9):** `select_utxos` uses the simplest possible strategy
  — walk the slice in the order given, adding each UTXO to the selection until
  the running total meets or exceeds the target, then stop. It's intentionally
  not "smart" (no attempt to minimize the number of UTXOs used, or to pick an
  exact match, or to avoid leaving dust-sized change). The trade-off: it's easy
  to reason about and matches the assignment's own test expectations exactly
  (`[70_000, 50_000]` targeting `90_000` selects *both*, since 70,000 alone falls
  short and the loop only stops once the running total clears the target).

  **A better real-world algorithm** would consider: (1) minimizing the number of
  inputs used (fewer inputs = smaller transaction = lower fee), (2) trying to
  find a combination that avoids creating a change output at all (an exact or
  near-exact match), and (3) avoiding selecting many small UTXOs together when a
  few large ones would do (to keep future transactions cheaper — spending many
  small UTXOs later costs more in fees than spending one large one). Real
  wallets (e.g. Bitcoin Core) use branch-and-bound search for this reason, only
  falling back to simpler strategies like this one when no good combination is
  found quickly.

- **Part 10 (optional transaction-state extension)** was not attempted, to keep
  focus on the required Parts 1–9 and make sure they're solid and fully tested
  first.

## Example output

```
Transaction v2 (locktime 0): 2 input(s), 2 output(s), total_in=120000 sats, total_out=118000 sats, fee=2000 sats
```

