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
