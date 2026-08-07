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

## Design notes

Describe any choices you made, including your UTXO-selection trade-offs and (if
attempted) the optional transaction-state extension.

## Example output

Paste the output of `cargo run` here once Part 8 is complete.

## written Solutions: 
1. A transaction input is a reference to an existing unspent output that is being spent. It proves ownership of that previous output and authorizes moving its value into a new transaction.
2. A transaction output is a new piece of bitcoin value created by a transaction. It specifies an amount and the conditions under which it can be spent by a future transaction.
3. A UTXO (Unspent Transaction Output) is an output that has been created by a transaction but has not yet been spent. Wallet balances are the sum of the `UTXOs` the wallet controls.
4. An outpoint uniquely identifies a specific UTXO using the pair `(txid, vout)`, where `txid` is the transaction hash and `vout` is the output index.
5. The fee is calculated as:

   `fee = total inputs − total outputs`

   Any value not assigned to an output becomes the `miner fee`.
6. Integers represent satoshis exactly, while floating-point numbers can introduce rounding errors. Financial software needs exact arithmetic, so satoshis are stored as integers.
7. It only reads the transaction to compute a sum. Borrowing with &self allows the method to inspect the transaction without taking ownership or modifying it.
8. Adding an input changes the transaction’s internal state by pushing a new value into the inputs vector, so a mutable reference is required.
9. `Ownership` of the input transfers to the transaction. The caller can no longer use that value unless it was cloned beforehand.
10. Result lets the caller handle errors gracefully(on their own terms), such as `insufficient funds or invalid fees`, without crashing the program. `panic!` is usually reserved for unrecoverable bugs.
11. The enum represents two distinct kinds of inputs with different data requirements. Exhaustive pattern matching forces the code to handle both regular and coinbase cases explicitly.
12. The trait provides a common way to obtain a value from different types. Generic code can work with any type implementing the trait instead of writing separate summation logic for inputs and outputs.

