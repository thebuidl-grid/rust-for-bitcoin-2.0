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

## Written Answers

### 1. What is a Bitcoin transaction input?

A transaction input points to a previous transaction output (UTXO) that the new transaction wants to spend. It contains information that identifies the previous output and provides the details needed to show that the coins can be spent.

### 2. What is a Bitcoin transaction output?

A transaction output specifies where a certain amount of Bitcoin should go. It creates a new UTXO and locks that amount to the recipient using a locking script.

### 3. What is a UTXO?

UTXO stands for Unspent Transaction Output. It is a Bitcoin output from an earlier transaction that has not been spent yet. These unspent outputs are what make up the Bitcoin that is currently available to be spent.

### 4. What does an outpoint identify?

An outpoint identifies a particular output from a particular Bitcoin transaction. It uses the transaction ID (`txid`) and the output's position (`vout`). The `vout` value starts at zero, so it tells us exactly which output in the transaction is being referenced.

### 5. How is a transaction fee calculated?

The transaction fee is the difference between the total value of the inputs and the total value of the outputs.

`fee = sum(inputs) - sum(outputs)`

For example, if a transaction uses 10,000 satoshis as inputs but only sends 9,500 satoshis in its outputs, the remaining 500 satoshis are the transaction fee. This fee is collected by the miner who includes the transaction in a block.

### 6. Why use integers rather than floating-point numbers for bitcoin amounts?

Bitcoin needs to be calculated exactly down to the smallest unit, which is one satoshi. Floating-point numbers can cause small rounding errors because some decimal values cannot be represented exactly in binary. Using an integer such as `u64` to store satoshis avoids these rounding problems and keeps the calculations accurate.

### 7. Why does `total_input_value()` borrow `self`?

`total_input_value()` only needs to look at the transaction's inputs and add their values together. It doesn't need to change anything in the transaction. Because of this, it can use a shared reference (`&self`) instead of taking ownership of the transaction.

This also means other parts of the program can still access the transaction after the function finishes.

### 8. Why does `add_input()` take `&mut self`?

`add_input()` changes the transaction by adding a new input to its `inputs` vector. Since the transaction is being modified, Rust requires a mutable reference (`&mut self`).

Rust also makes sure that there cannot be multiple mutable references to the same data at the same time, which helps prevent data races and other memory-related bugs.

### 9. What happens when an input is moved into a transaction?

When an `InputKind` is passed into `add_input()`, ownership of that value is transferred to the transaction. The transaction then owns the input because it stores it in its `inputs` vector.

The original variable can no longer be used after the move because it no longer owns the value. This is one of the main ways Rust's ownership system manages memory safely.

### 10. Why is `Result` preferable to `panic!` for validation failures?

Validation errors are normally expected situations rather than bugs in the program. For example, a transaction might fail because it does not have enough inputs to cover its outputs.

Using `Result` allows the program to handle these failures properly. The caller can decide whether to display an error, retry the operation, or take some other action. Using `panic!` would stop the program instead, which is usually unnecessary for normal validation failures.

### 11. How do enums help model regular and coinbase inputs?

Regular inputs and coinbase inputs contain different types of information. A regular input references a previous transaction output, while a coinbase input contains information such as the block height.

An enum allows both types to be represented by the same `InputKind` type while keeping their different fields separate. Using `match` also means Rust can check that both variants are handled whenever the input type is processed.

### 12. How does the `BitcoinValue` trait reduce duplication?

Both `TxOutput` and `InputKind` have a Bitcoin value associated with them, even though that value may be stored differently internally.

The `BitcoinValue` trait gives both types a common `value()` method. This means functions such as `total_input_value()` can simply ask each input for its value instead of having to write separate logic for every input variant.

This makes the code shorter, easier to understand, and easier to extend later.

## Part 7 — Ownership Compiler Error

The following error happens when an input is used after it has already been moved into a transaction:

```text
error[E0382]: borrow of moved value: `input`
  --> src/main.rs:42:20
   |
38 |     let input = InputKind::Regular { ... };
   |         ----- move occurs because `input` has type `InputKind`, which does not implement the `Copy` trait
39 |     transaction.add_input(input);
   |                           ----- value moved here
40 |
41 |     println!("{input}");
   |                ^^^^^ value borrowed here after move
```

### Explanation

The error happens because `add_input()` takes the `InputKind` by value, which means it takes ownership of it. When `input` is passed to the transaction, the transaction becomes the new owner of that value.

Because the original variable no longer owns the input, Rust does not allow it to be used afterward. In this case, trying to print `input` causes the compiler error.

This is part of Rust's ownership system. A value normally has one owner at a time, and once ownership has been moved, the previous owner cannot use the value anymore.

## Design Notes

### Validation Order

The validation checks are performed in the order required by the assignment. The transaction is first checked for empty inputs or outputs, followed by zero-value outputs, fee balance, and finally the rules for the different input types.

Keeping the checks in this order makes the behavior predictable and ensures that the appropriate error is returned when more than one problem exists.

### UTXO Selection

The basic UTXO selection algorithm goes through the available UTXOs in their existing order and keeps selecting them until there is enough value to cover the target amount.

This approach is simple and predictable, but it is not necessarily the most efficient. It might select more UTXOs than needed, which can make the resulting transaction larger and potentially increase the transaction fee.

A more advanced implementation could use strategies such as largest-first selection or branch-and-bound to try to find a better combination of UTXOs. These approaches can reduce the number of inputs and the amount of unnecessary change, although they are more complicated to implement.

### Optional State Machine (Part 10)

The optional state machine was not implemented. The current implementation focuses on representing Bitcoin transactions and validating their contents rather than tracking the different stages of a transaction's lifecycle.


## Example output

```text
Transaction v2 (locktime 0): 2 inputs, 2 outputs, total input 120000 sats, total output 118000 sats, fee 2000 sats
```
