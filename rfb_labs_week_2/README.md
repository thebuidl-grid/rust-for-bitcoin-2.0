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

### Part 7 ownership experiment

```text
error[E0382]: borrow of moved value: `output`
  --> src/main.rs:...
   |
   | transaction.add_output(output);
   |                        ------ value moved here
   | println!("{}", output.recipient);
   |                ^^^^^^^^^^^^^^^^ value borrowed here after move
```

`add_output` takes ownership of its `TxOutput` argument and moves it into the
transaction's `outputs` vector. The original variable can therefore no longer be
used. Borrowing it before the move, or reading it through the transaction after the
move, is valid.

1. **What is a Bitcoin transaction input?** An input spends a previously created
   UTXO. It points to that output and proves that the spender is allowed to use it;
   this simplified model stores its value directly as well.
2. **What is a Bitcoin transaction output?** An output assigns a number of satoshis
   to a recipient under a particular spending condition. It can later become an
   input to another transaction.
3. **What is a UTXO?** A UTXO is an unspent transaction output: bitcoin value that
   is available to be spent because no later transaction has consumed it yet.
4. **What does an outpoint identify?** An outpoint identifies one exact output from
   an earlier transaction using its transaction ID (`txid`) and output position
   (`vout`).
5. **How is a transaction fee calculated?** The fee is the total value of all
   inputs minus the total value of all outputs. In this project, the subtraction is
   checked so a transaction cannot underflow when outputs are greater than inputs.
6. **Why use integers rather than floating-point numbers for bitcoin amounts?**
   Satoshis are indivisible units. Integers represent them exactly, while floating
   point numbers can introduce rounding errors.
7. **Why does `total_input_value()` borrow `self`?** It only reads input values to
   calculate a total. Borrowing lets the transaction remain usable after the method
   call and avoids moving or cloning its inputs.
8. **Why does `add_input()` take `&mut self`?** Adding an input changes the
   transaction's input vector, so it needs exclusive mutable access while keeping
   the transaction itself owned by the caller.
9. **What happens when an input is moved into a transaction?** Ownership transfers
   from the caller into the transaction's vector. The old variable cannot be used
   afterward unless the type implements and is explicitly cloned.
10. **Why is `Result` preferable to `panic!` for validation failures?** Invalid
    transaction data is an expected condition, not a programmer crash. `Result`
    gives the caller a specific error that it can display, retry, or handle safely.
11. **How do enums help model regular and coinbase inputs?** `InputKind` makes an
    input exactly one valid form. Pattern matching then requires code to consider
    both forms and their different fields.
12. **How does the `BitcoinValue` trait reduce duplication?** It supplies one
    shared `value()` interface for outputs and both input variants, so code can ask
    for a bitcoin amount without repeating variant-specific logic at every call
    site.

## Design notes

`select_utxos` deliberately chooses UTXOs in their supplied order. It is easy to
understand, deterministic, and returns references instead of copying UTXOs, but it
does not minimize change or the number of inputs. A more sophisticated wallet could
use a branch-and-bound search to find an exact match or a smaller-change combination;
that trades predictability and implementation simplicity for potentially better fees
and privacy.

For the optional state exercise, `TransactionLifecycle` owns a transaction and tracks
the Created, Validated, Signed, Broadcast, Confirmed, and Rejected states. Its state
field is private, so callers must use transition methods. Each transition consumes the
lifecycle and returns it only when the current state permits that transition.

## Example output

```text
Transaction v2 (locktime 0): 2 input(s), 2 output(s), inputs: 120000 sats, outputs: 118000 sats, fee: 2000 sats
```
