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
   A bitcoin transaction input is a reference to a previous transaction output that you want to spend, it is specifically an outpoint (txid + vout) that points to a UTXO plus the unlocking data that proves you are authorized to spend it. `InputKind::Regular` demonstrates this.
2. What is a Bitcoin transaction output?
   A bitcoin transaction output is where value is created and locked to a spending condition. It basically declares "this amount is spendable by whoever satisfies this condition", and it sits as a UTXO until an input in the future references it. `TxOutput` struct maps to that.
3. What is a UTXO?
   A UTXO (Unspent Transaction Output) is a transaction output that has not been spent yet, it hasn't been added as an input to a transaction. It is an amount locked to a spending condition, available to be referenced in the future to an input via its txid and vout
4. What does an outpoint identify?
   An outpoint identifies one UTXO by `txid + vout`, where txid is the transaction that created the output and vout is the index of that output in the transaction's list of outputs.
5. How is a transaction fee calculated?
   A transaction fee is calculated as the difference between the total input value and total output value `fee = total_inputs - total_outputs`. It was implemented in our code as `total_input_value() - total_output_value()` computed via `checked_sub`
6. Why use integers rather than floating-point numbers for bitcoin amounts?
   Bitcoin amounts are tracked as integers, in satoshis, because floating point numbers cannot accurately represent decimal fractions exactly, and money needs exact equality/comparison, satoshis as u64 fixes that completely, every value is a whole exact integer and addition and subtraction operations provide a uniform result.
7. Why does `total_input_value()` borrow `self`?
   `total_input_value()` borrows `self` because it only needs to read fields to compute a sum. It doesn't mutate anything or needs ownership, so there is no need to move self and borrowing does the job perfectly.
8. Why does `add_input()` take `&mut self`?
   `add_input()` takes `&mut self` because it mutates self.inputs by pushing into the `Vec`, so it needs write access.
9. What happens when an input is moved into a transaction?
   Ownership of the `InputKind` value transfers to the `Vec` inside `Transaction`, the original binding becomes invalid and the compiler prevents further use of it.

```text
❯ cargo build

   Compiling rfb-labs-week-2 v0.1.0 (/Users/hakeem/Desktop/26/rust-for-bitcoin-2.0/rfb_labs_week_2)
error[E0382]: borrow of moved value: `scratch_output`
  --> src/main.rs:43:20
   |
37 |     let scratch_output = TxOutput {
   |         -------------- move occurs because `scratch_output` has type `TxOutput`, which does not implement the `Copy` trait
...
42 |     transaction.add_output(scratch_output);
   |                            -------------- value moved here
43 |     println!("{}", scratch_output.value); // trying to use it after it moved
   |                    ^^^^^^^^^^^^^^^^^^^^ value borrowed here after move

For more information about this error, try `rustc --explain E0382`.
error: could not compile `rfb-labs-week-2` (bin "rfb-labs-week-2") due to 1 previous error
```

This happens because `add_output` takes `scratch_output: TxOutput` by value, not by reference (&) — calling it moves `scratch_output` into `transaction.outputs`, so `scratch_output` no longer refers to valid data in the caller's scope afterward.

10. Why is `Result` preferable to `panic!` for validation failures?
    Validation failures are expected, `panic!` aborts the whole program and is reserved for bugs or violations, not for expected bad input which are safely handled using `Result`.
11. How do enums help model regular and coinbase inputs?
    `InputKind` in our code makes it impossible to represent an invalid state for example, a coinbase input with a `previous_output` and it forces exhaustive match handling everywhere it is used. The compiler enforces handling all variants
12. How does the `BitcoinValue` trait reduce duplication?
    `value_in_btc()` is defined once as a default method in the `BitcoinValue` trait, implementation of `BitcoinValue` automatically provides a BTC-conversion value for free once implemented.

## Design notes

Describe any choices you made, including your UTXO-selection trade-offs and (if
attempted) the optional transaction-state extension.

The `select_utxos` algorithm implmented is simplest-possible and not fee-optimal, also not privacy-optimal. Real wallets would use smarter selections to reduce fees and avoid linking UTXOs unnecessarily. Part 10 was not attempted, the base `Transaction` model has no lifecycle state

## Example output

Paste the output of `cargo run` here once Part 8 is complete.

```text
❯ cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.17s
     Running `target/debug/rfb-labs-week-2`
Transaction v2 locktime=0 inputs=2 outputs=2 total_input=120000 sats total_output=118000 sats fee=2000 sats
```
