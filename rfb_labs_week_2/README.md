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
 
1. **What is a Bitcoin transaction input?** It is a reference to value being spent —
   either a pointer (an `OutPoint`) to a previous transaction's output, along with the
   value it carries and a sequence number, or, for a coinbase input, the newly minted
   block reward. It is the "money coming in" side of a transaction.
2. **What is a Bitcoin transaction output?** It is a destination for value: an amount
   in satoshis, a recipient, and an output type describing the locking script shape
   (e.g. P2PKH, P2WPKH, P2TR, or a data-carrying `OP_RETURN`). Outputs become the
   UTXOs a future transaction can spend.
3. **What is a UTXO?** An Unspent Transaction Output — an output from some past
   transaction that hasn't yet been consumed as an input elsewhere. Wallets track
   their UTXOs to know what funds they can spend next.
4. **What does an outpoint identify?** A specific output of a specific previous
   transaction — the pair `(txid, vout)`, i.e. "output number `vout` of the
   transaction with this txid."
5. **How is a transaction fee calculated?** It's the total input value minus the
   total output value. Nothing is charged explicitly; whatever isn't sent to an
   output is implicitly paid to the miner as the fee.
6. **Why use integers rather than floating-point numbers for bitcoin amounts?**
   Floating-point arithmetic introduces rounding error, and satoshis are already the
   smallest indivisible unit, so representing value as `u64` satoshis keeps every
   calculation exact — critical when money is on the line.
7. **Why does `total_input_value()` borrow `self`?** It only needs to read the
   transaction's inputs to sum their values; it doesn't need to own or mutate the
   transaction, so an immutable borrow (`&self`) is the minimal permission needed and
   lets the caller keep using the transaction afterward.
8. **Why does `add_input()` take `&mut self`?** It mutates the transaction by pushing
   a new input into its `inputs` vector, so it needs a mutable, exclusive borrow.
9. **What happens when an input is moved into a transaction?** Ownership of the
   `InputKind` value transfers into the `Vec<InputKind>` inside the transaction. The
   caller's original variable is no longer valid — the compiler considers it moved,
   and using it again is a compile-time error (see below).
10. **Why is `Result` preferable to `panic!` for validation failures?** Invalid
    transaction data (empty inputs, mismatched totals, bad txids) is an expected,
    recoverable condition, not a bug. `Result` forces the caller to explicitly handle
    the failure instead of crashing the whole program, and it lets calling code
    decide what to do (reject, retry, report).
11. **How do enums help model regular and coinbase inputs?** `InputKind` expresses
    that an input is *either* a regular spend *or* a coinbase reward, never both,
    directly in the type system. Pattern matching on the enum forces every call site
    to handle both variants, so it's impossible to accidentally treat a coinbase
    input like a regular one (or forget to handle one case) without a compiler error.
12. **How does the `BitcoinValue` trait reduce duplication?** It defines `value()`
    once as the required method and derives `value_in_btc()` from it as a default
    method. `TxOutput` and `InputKind` each only need to implement `value()`; the
    sats-to-BTC conversion logic is written once on the trait instead of being
    copy-pasted into every implementing type.
### Ownership experiment (Part 7)
 
Moving an `InputKind` into `transaction.add_input(input)` and then trying to use
`input` again afterward fails to compile:
 
```text
error[E0382]: borrow of moved value: `input`
  --> examples/ownership_experiment.rs:14:15
   |
5  |     let input = InputKind::Regular {
   |         ----- move occurs because `input` has type `InputKind`, which does not implement the `Copy` trait
...
11 |     transaction.add_input(input);
   |                           ----- value moved here
...
14 |     println!("{input:?}");
   |               ^^^^^^^^^ value borrowed here after move
```
 
`InputKind` doesn't implement `Copy` (it owns a `String` inside `OutPoint`), so
passing it by value into `add_input` transfers ownership rather than duplicating it.
Once the transaction owns the input, the original `input` binding is no longer valid,
and the borrow checker rejects any later use of it. This is exactly what we want: it
guarantees a given input can't accidentally be attached to two transactions, or read
from after it's already been "spent" into one.
 
## Design notes
 
- **UTXO selection (`select_utxos`):** the required algorithm is a simple greedy scan
  in slice order — take UTXOs one at a time until the running total meets the target,
  stopping as soon as it's met. It's easy to reason about and cheap (`O(n)`), and it
  matches the order the caller already chose to present UTXOs in.
- **Bonus — a better selection algorithm:** slice-order selection can leave a wallet
  with lots of small "dust" UTXOs, or overshoot the target by a large margin and
  create a big change output that hurts privacy. A better real-world approach is
  Bitcoin Core's **Branch and Bound** selection: it searches for a subset of UTXOs
  that sums *exactly* (or very close) to the target plus fee, avoiding a change
  output entirely when possible; when no exact match exists it falls back to a
  strategy like largest-first or single-random-draw. That trades a bit more
  computation for smaller transactions (lower fees) and better long-term UTXO-set
  health.
- **Errors:** `TransactionError` implements `std::error::Error` and a descriptive
  `Display`, and every validation failure returns a `Result` instead of panicking, so
  malformed input data is always recoverable by the caller.
- **Part 10 (state machine) was not attempted** in this submission.
## Example output
 
```
Transaction v2 (locktime 0)
  inputs: 2
  outputs: 2
  total input: 120000 sats
  total output: 118000 sats
  fee: 2000 sats
```
