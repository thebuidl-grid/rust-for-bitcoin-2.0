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
A bitcoin transaction input is a reference to a specific UTXO(Unspent transaction outputs) or unspent bitcoins that you are willing to spend in a transaction 

2. What is a Bitcoin transaction output?
A bitcoin transaction output is the amount of btc sent to a person in a transaction. An output has a locking script defining who can spend it. 

3. What is a UTXO?
A UTXO which means Unspent Transaction Output is the amount of bitcoin sitting in your wallet that has not been spent

4. What does an outpoint identify?
An outpoint identifies your specific output. What this means is that every output you have is tied under an index in a transaction id.  So an outpoint tracks your output by combining your transaction id and output index, like this: [TXID][Output Index].

5. How is a transaction fee calculated?
A transaction fee is calculated by subtracting sum of outputs from sum of inputs. The miner keeps whatever input value is not assigned to any output. 

6. Why use integers rather than floating-point numbers for bitcoin amounts?
This is because floating-point numbers have rounding errors that could cause satoshi 
amounts to be calculated incorrectly, since 1 BTC = 100,000,000 satoshis, all amounts fit in integers exactly with no rounding 
needed.

7. Why does `total_input_value()` borrow `self`?
This is because `total_input_value()` is a getter/reading function, so it does not want to consume or take ownership of an instance of  the `Transaction` struct.`

8. Why does `add_input()` take `&mut self`?
This is because `add_input()` wants to update an instance of the `Transaction` struct but does not want to take ownership of it. That is why it is referncing and mutating, meaning that we can modify the transaction without consuming it, so the caller can continue using the transaction after the call.

9. What happens when an input is moved into a transaction?
Now, once the `input` value is moved into the transaction's inputs' vector, the transaction now owns the input. When `input` is moved, we add either of it's variants: `Regular` or `Coinbase`. The moment we add it, we've made change to the instance of the `Transaction` struct but still referencing it. 

10. Why is `Result` preferable to `panic!` for validation failures?
It is because `Result` handles errors for validation failures, but when you use `panic!`, your whole program crashes. With `Result`, you handle both edge cases where either the operation passes or it fails.

11. How do enums help model regular and coinbase inputs?
it is beacuse Inputs can either be of two types, either it is `Regular` or it is `Coinbase`. That's what `enums` aim to do. They handle the state of something using variants which in this case is `Regular` or `Coinbase`. There can be only one kind of inputs at a time, it can never be both. And also the compiler forces you to handle both variants when you match on `InputKind`.

12. How does the `BitcoinValue` trait reduce duplication?
Without the `BitcoinValue` trait, you's need need seperate functions to get the value of a `TxOutput` and an `InputKind`. The trait defines one value() method that both types implement, so any code that just needs a value can work with either type through the trait. No duplication of logic.


## Design notes
For UTXO selection (Part 9), I selected UTXOs in slice order, accumulating their values until the target amount is reached. This approach is simple and predictable but not optimal — it may select more UTXOs than necessary, which increases transaction size and fees. A better algorithm would sort UTXOs by value descending and pick the smallest set that covers the target, or use an exact-match algorithm like Bitcoin Core's Branch and Bound to avoid creating change outputs when possible.

For validation (Part 5), I check rules in order of cheapest to most expensive: empty inputs/outputs first, then zero-value outputs, then fee calculation, then coinbase mixing, and finally TXID validity. This avoids unnecessary work when an early rule fails.

The `InputKind` enum models the two fundamentally different input types — regular and coinbase — as distinct variants. This forces any code that handles inputs to explicitly deal with both cases via `match`, making it impossible to accidentally treat a coinbase input as a regular one or miss a variant.


## Example output
Paste the output of `cargo run` here once Part 8 is complete.
```bash
henry-peters@henry-peters-Latitude-3340:~/Desktop/rust-for-bitcoin-2.0/rfb_labs_week_2$ cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.03s
     Running `target/debug/rfb-labs-week-2`
Transaction { version: 2, locktime: 0, inputs: 2, outputs: 2, total_input: 120000 sats, total_output: 118000 sats, fee: 2000 sats }
```
