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

## Ownership and Borrowing Experiment

### Part 7 Compiler Error Output

```text
error[E0382]: borrow of moved value: `output`
  --> src/main.rs:13:34
   |
 7 |     let output = rfb_labs_week_2::TxOutput {
   |         ------ move occurs because `output` has type `TxOutput`, which does not implement the `Copy` trait
...
12 |     transaction.add_output(output);
   |                            ------ value moved here
13 |     println!("Output value: {}", output.value);
   |                                  ^^^^^^^^^^^^ value borrowed here after move
```

### Explanation

- **What value was moved:** The variable `output` of type `TxOutput` was moved when passed by value into `transaction.add_output(output)`.
- **Why Rust rejected later use:** `TxOutput` contains a `String` (`recipient`) and does not implement the `Copy` trait. When `output` was passed by value to `add_output(self, output: TxOutput)`, ownership was transferred to `transaction.outputs`. Consequently, `output` in `main()` became invalid/uninitialized, making any subsequent attempt to read or borrow `output.value` a compile-time error.
- **How borrowing changes the situation:** Borrowing (`&transaction` or `&output`) creates a reference pointing to the existing data without moving ownership. The underlying memory remains owned by its original container, enabling multiple components to inspect the data safely under Rust's borrowing rules.

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

```text
Transaction v2 (locktime: 0)
  Inputs (2): total 120000 sats
    [0] Regular(1111111111111111111111111111111111111111111111111111111111111111:0, 70000 sats, seq: 0xffffffff)
    [1] Regular(2222222222222222222222222222222222222222222222222222222222222222:1, 50000 sats, seq: 0xffffffff)
  Outputs (2): total 118000 sats
    [0] 90000 sats -> bc1qreceiver (P2wpkh)
    [1] 28000 sats -> bc1qsender (P2wpkh)
  Fee: 2000 sats
```
