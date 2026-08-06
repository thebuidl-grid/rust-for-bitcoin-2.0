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
It contains the value that is going to be spent in a transaction. A transaction is going to use the necessary values to pay the outputs + fees. A input can only be spent using the a private key. An input contains the previous transaction id (txid), the vout (index of output used in the previous transaction), ScriptSig that unlocks the output to be spent.

2. What is a Bitcoin transaction output?
A output is the amount being sent and the ScriptPubKey, this script defines what must be met to spend this output later in the future. It is a locking mecanism. The ScriptPubKey can be,  for example: P2pkh, P2wpkh, P2tr.
When a transaction is made, the inputs are used and new outputs are created. In the next transaction the outputs are going to be inputs.

3. What is a UTXO?
UTXO stands for Unspent Transaction Outpout.
They are the value that can be spent in a transaction. When a transaction is created the inputs are used and new outputs are created. When any of these outputs were not spent in a transaction they are UTXO. 

4. What does an outpoint identify?
An outpoint identifies the transaction (txid) that created the output and the output’s index (vout) within that transaction . It specify the output among the transaction’s outputs.

5. How is a transaction fee calculated?
The transaction fee is calculated  by the sum of the inputs values minus the sum of the outputs values.

6. Why use integers rather than floating-point numbers for bitcoin amounts?
Because rounding problems can happen when working with floats.Different nodes and hardware can handle floats in diverse ways, which could create consensus problems. It’s safe to work with integers (sats) and avoid those issues.

7. Why does `total_input_value()` borrow `self`?
8. Why does `add_input()` take `&mut self`?
9. What happens when an input is moved into a transaction?
10. Why is `Result` preferable to `panic!` for validation failures?

11. How do enums help model regular and coinbase inputs?
12. How does the `BitcoinValue` trait reduce duplication?

## Design notes

Describe any choices you made, including your UTXO-selection trade-offs and (if
attempted) the optional transaction-state extension.

## Example output

Paste the output of `cargo run` here once Part 8 is complete.
