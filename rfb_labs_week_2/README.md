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

1. **What is a Bitcoin transaction input?**
   A Bitcoin transaction input is a reference to a previous, unspent transaction output (UTXO) that is being spent in the current transaction. In a real Bitcoin transaction, it also contains a cryptographic signature (witness/unlocking script) that proves ownership of those funds.

2. **What is a Bitcoin transaction output?**
   A Bitcoin transaction output defines the recipient of the funds and the amount of satoshis being transferred. It locks the funds to a specific address or script (locking script/scriptPubkey) which must be satisfied by a future input to spend those funds.

3. **What is a UTXO?**
   UTXO stands for Unspent Transaction Output. It is an output from a previous transaction that has not yet been spent by any input. The set of all UTXOs in the Bitcoin network represents the current ledger of spendable coins.

4. **What does an outpoint identify?**
   An outpoint identifies a specific transaction output being spent. It consists of the transaction ID (`txid`) where the output was created, and the index (`vout`) of that output within the transaction's outputs list.

5. **How is a transaction fee calculated?**
   The transaction fee is calculated as the difference between the sum of all input values and the sum of all output values: `Fee = Total Input Value - Total Output Value`. There is no explicit fee field in a Bitcoin transaction; the remainder is collected by miners.

6. **Why use integers rather than floating-point numbers for bitcoin amounts?**
   Floating-point numbers can introduce rounding and precision errors due to how they represent decimal fractions in binary. Because financial ledgers require absolute precision, Bitcoin uses 64-bit integers (`u64`) to represent amounts in satoshis (the smallest unit, `1 BTC = 100,000,000 satoshis`) to avoid any precision loss or rounding inconsistencies.

7. **Why does `total_input_value()` borrow `self`?**
   It borrows `self` (specifically via a shared reference `&self`) because it only needs read-only access to inspect the transaction inputs and compute their sum. It does not need to modify the transaction or take ownership of it.

8. **Why does `add_input()` take `&mut self`?**
   It takes `&mut self` because it modifies the internal state of the `Transaction` by appending a new input to the `inputs` vector. This requires exclusive, mutable access to the transaction.

9. **What happens when an input is moved into a transaction?**
   When an input (`InputKind`) is passed by value to `add_input`, ownership of the input value is transferred (moved) from the caller to the transaction structure. The caller can no longer use the input variable unless it is returned or cloned, enforcing Rust's move semantics.

10. **Why is `Result` preferable to `panic!` for validation failures?**
    Using `Result` allows validation errors (such as invalid TXIDs, outputs exceeding inputs, or empty inputs) to be handled gracefully by the calling application (e.g. prompt the user to correct the inputs) without terminating the entire program. `panic!` crashes the program immediately, which is bad practice for user-facing or robust network applications.

11. **How do enums help model regular and coinbase inputs?**
    Enums allow us to represent a value that can be one of several different variants. Regular inputs and Coinbase inputs have different data structures (e.g. regular inputs spend previous outpoints, while coinbase inputs contain block heights). By using an enum (`InputKind`), we can hold either variant in the same `Vec` and use pattern matching (`match`) to force type-safe handling of both cases.

12. **How does the `BitcoinValue` trait reduce duplication?**
    The `BitcoinValue` trait defines a shared interface (`value()` and `value_in_btc()`) for any type representing a Bitcoin value. By implementing this trait for both `TxOutput` and `InputKind`, we can write generic functions or reuse calculations (like converting satoshis to BTC) without repeating the logic for each type.

### Part 7 Ownership Compiler Error

```text
error[E0507]: cannot move out of index of `Vec<TxOutput>`
   --> src\transaction.rs:164:17
    |
164 |     let first = transaction.outputs[0];
    |                 ^^^^^^^^^^^^^^^^^^^^^^ move occurs because value has type `TxOutput`, which does not implement the `Copy` trait
    |
note: if `TxOutput` implemented `Clone`, you could clone the value
   --> src\transaction.rs:14:1
    |
 14 | pub struct TxOutput {
    | ^^^^^^^^^^^^^^^^^^^ consider implementing `Clone` for this type
...
164 |     let first = transaction.outputs[0];
    |                 ---------------------- you could clone this value
help: consider borrowing here
    |
164 |     let first = &transaction.outputs[0];
    |                 +

error[E0515]: cannot return value referencing local variable `first`
   --> src\transaction.rs:165:5
    |
165 |     Some(&first)
    |     ^^^^^------^
    |     |    |
    |     |    `first` is borrowed here
    |     returns a value referencing data owned by the current function
```

**Explanation:**
The error occurred because we attempted to assign `transaction.outputs[0]` to the local variable `first`. Since `TxOutput` does not implement the `Copy` trait, this assignment attempts to move ownership of the output out of the `Vec` inside the `Transaction` struct. Since the transaction is passed by a shared borrow (`&Transaction`), we cannot take ownership of its contents. Additionally, returning `Some(&first)` references a local variable `first` which goes out of scope at the end of the function, which violates Rust's borrowing rules (dangling reference). The correct approach is to borrow elements directly from the `Vec` using reference iteration or indexing (`&transaction.outputs[0]`), returning a reference that links its lifetime directly to the input `transaction` reference.

## Design notes

We implemented the UTXO selection algorithm by iterating through the available UTXOs in slice order and adding them to the selected list until the target amount is met.
**UTXO Selection Trade-offs:**
The basic slice-order selection (first-in-first-out / sequential) is simple and deterministic. However, it has some trade-offs:
1. **Privacy/Traceability:** Spending UTXOs sequentially can link separate addresses and transaction histories, reducing transaction privacy.
2. **Fee Optimization:** In real Bitcoin, each input added to a transaction increases the size of the transaction (in vbytes), which increases the required fee. A selection algorithm like "Branch and Bound" or "Largest First" can minimize the number of inputs needed, thereby reducing transaction fee costs.
3. **UTXO Pool Consolidation/Fragmentation:** Selecting many small UTXOs helps clean up dust (consolidation) but increases fees. Selecting only a few large UTXOs avoids high fees but leaves many small dust inputs behind.

## Example output

```text
Transaction (v2, locktime: 0)
  Inputs (count: 2):
    - Regular input spending aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:0 (70000 sats, seq: 4294967295)
    - Regular input spending bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb:1 (50000 sats, seq: 4294967295)
  Outputs (count: 2):
    - 90000 sats to bc1qreceiver (P2wpkh)
    - 28000 sats to bc1qsender (P2wpkh)
  Total Input:  120000 sats
  Total Output: 118000 sats
  Calculated Fee: 2000 sats
```
