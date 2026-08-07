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

#### Part 1–2: Data model

`InputKind` is an enum because a Bitcoin transaction input can be one of two different types: a regular input that spends a previous UTXO, or a coinbase input that creates new coins as a mining reward. Since an input cannot be both at the same time, an enum models this relationship naturally.

Using `match` on `InputKind` forces Rust to handle every possible variant. If a new input type is added in the future, the compiler will require every `match` expression to handle it, reducing bugs and making the code safer.

#### Ownership compiler error (Part 7)

```text
error[E0382]: borrow of moved value: `input`
```

This happened because ownership of `input` was moved into the transaction when it was added to the `inputs` vector. After a value is moved in Rust, the original variable can no longer be used. Rust enforces this rule to prevent multiple owners of the same data and to guarantee memory safety.


1. What is a Bitcoin transaction input?

```A Bitcoin transaction input is a reference to an existing unspent transaction output (UTXO). It proves which coins are being spent in a new transaction.```

2. What is a Bitcoin transaction output?

```A transaction output is a new amount of bitcoin created by a transaction. It specifies the value and the recipient who can spend it in the future.```

3. What is a UTXO?

```A UTXO (Unspent Transaction Output) is an output that has not yet been spent. Bitcoin wallets keep track of their UTXOs because they represent the spendable balance.```

4. What does an outpoint identify?

```An outpoint uniquely identifies a specific UTXO by combining the transaction ID (txid) with the output index (vout).```

5. How is a transaction fee calculated?

```The transaction fee is calculated by subtracting the total value of all outputs from the total value of all inputs. Fee = Total Inputs - Total Outputs```

6. Why use integers rather than floating-point numbers for bitcoin amounts?

```Integers avoid rounding errors. Bitcoin uses satoshis, the smallest unit, so integer arithmetic keeps calculations accurate and deterministic.```

7. Why does `total_input_value()` borrow `self`?

```It only needs to read the transaction's inputs, not change them. Borrowing with &self allows the method to access the data without taking ownership.```

8. Why does `add_input()` take `&mut self`?

```Because it modifies the transaction by adding a new input to the inputs vector. Mutable borrowing allows the transaction to be changed safely.```

9. What happens when an input is moved into a transaction?

```Ownership of the input is transferred to the transaction. The original variable can no longer be used unless it implements Copy, which prevents accidental reuse of the same value.```

10. Why is `Result` preferable to `panic!` for validation failures?

```Validation failures are expected situations, not program crashes. Returning a Result lets the caller handle errors gracefully instead of terminating the application.```

11. How do enums help model regular and coinbase inputs?

```Enums allow different kinds of transaction inputs to be represented with one type. Pattern matching ensures both regular inputs and coinbase inputs are handled explicitly.```

12. How does the `BitcoinValue` trait reduce duplication?

```The trait provides a common way to get the value of different Bitcoin-related types. This avoids writing separate functions for inputs and outputs that do the same thing.```

## Design notes

Describe any choices you made, including your UTXO-selection trade-offs and (if
attempted) the optional transaction-state extension.

```For UTXO selection, I used a simple algorithm that selects UTXOs in the order they appear until the target amount is reached. This approach is easy to understand and implement, although it is not always the most efficient because it may use more inputs than necessary or produce larger change outputs. More advanced wallets often use coin selection algorithms that optimize for lower fees, improved privacy, or reduced change.```

## Example output

Paste the output of `cargo run` here once Part 8 is complete.
```text
Transaction
  Version: 2
  Locktime: 0
  Inputs: 2
  Outputs: 2
  Total Input: 120000 sats
  Total Output: 118000 sats
  Fee: 2000 sats
```

