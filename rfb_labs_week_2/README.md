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
ans:m A reference to an unspent transaction output (`OutPoint`) from a previous transaction, carrying value into the new transaction.
2. What is a Bitcoin transaction output?
ans: A specified satoshi amount along with locking script/recipient conditions defining who can spend those funds next.
3. What is a UTXO?
ans: Unspent Transaction Output: an output of a past transaction that has not yet been spent as an input in another transaction.
4. What does an outpoint identify?
ans: A specific output from a previous transaction, composed of a 32-byte `txid` string and an output index (`vout`).
5. How is a transaction fee calculated?
ans: `Fee = Total Input Value - Total Output Value`.
6. Why use integers rather than floating-point numbers for bitcoin amounts?
ans: Floating-point types (`f64`) introduce IEEE-754 precision issues and non-deterministic rounding errors. Satoshis represented as `u64` integers ensure exact arithmetic across all platforms.
7. Why does `total_input_value()` borrow `self`?
ans: It only inspects internal input values to sum them; borrowing `&self` avoids taking ownership or mutating the transaction.
8. Why does `add_input()` take `&mut self`?
ans: It mutates the `Transaction` struct by modifying its internal `inputs: Vec<InputKind>` collection.
9. What happens when an input is moved into a transaction?
ans: Ownership transfers from the caller to the `Transaction` instance. The caller can no longer access or reuse the moved `InputKind`.
10. Why is `Result` preferable to `panic!` for validation failures?
ans: Invalid inputs or outputs are expected runtime error cases. Returning `Result` allows callers to handle validation errors safely without crashing the execution process.
11. How do enums help model regular and coinbase inputs?
ans: `InputKind` captures structural differences (`OutPoint` vs `block_height`). Rust pattern matching forces exhaustive handling of both variants across calculations and validation checks.
12. How does the `BitcoinValue` trait reduce duplication?
ans: It unifies value extraction logic across different types (`TxOutput`, `InputKind`) and provides shared default methods like `value_in_btc()`.

## Design notes

We implemented sequential coin selection for `select_utxos` iterating over a borrowed slice `&[Utxo]`. This satisfies target requirements while avoiding vector allocations or cloning.

## Example output

elsuraj@El-suraj:~/rust-for-bitcoin-2.0/rfb_labs_week_2$ cargo run 
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/rfb-labs-week-2`
Transaction Summary (v2, locktime: 0)
  Inputs (2): Total 120000 sats
  Outputs (2): Total 118000 sats
  Fee: 2000 sats