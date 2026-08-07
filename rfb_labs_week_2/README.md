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

### 1. What is a Bitcoin transaction input?

A Bitcoin transaction input is an output that is being spent in a transaction. It is a reference to a previous unspent transaction output (UTXO) that is being spent. It identifies which UTXO is being consumed by including the previous transaction ID and the output index (vout). Each input can only be spent once, and the transaction creator proves they have the authority to spend it through a signature.

### 2. What is a Bitcoin transaction output?

A Bitcoin transaction output is a package of bitcoins created in a bitcoin transaction. When one create multiple outputs in a transaction, where each output contains an amount of bitcoin and a lock on it. A future transaction can then spend these outputs (as inputs) by unlocking them, and create new outputs with new locks on them.  OR
A Bitcoin transaction output is a record that specifies an amount of bitcoin and the conditions required to spend it in the future. Once created, it becomes a UTXO until it is spent.

### 3. What is a UTXO?

A UTXO (Unspent Transaction Output) is a transaction output that has not yet been spent. UTXOs are the fundamental unit of value in Bitcoin: they represent coins that can be spent in future transactions. The total spendable balance of a Bitcoin address is the sum of all UTXOs that can be unlocked with its private key.


### 4. What does an outpoint identify?

An outpoint uniquely identifies a specific output from a specific transaction. It consists of two parts: a transaction ID (TXID, the hash of the entire transaction) and a vout (output index). This allows any future transaction to precisely reference which UTXO it intends to spend.

### 5. How is a transaction fee calculated?

A UTXO (Unspent Transaction Output) is a transaction output that has not yet been spent. It represents bitcoin that is available to be used as an input in a future transaction.
`Transaction Fee = Total Inputs − Total Outputs`
Any remaining amount is treated as the miner's fee.
It represents the amount of satoshi that are "burned" (not included in any output). Miners include transactions with higher fees first, so users can pay higher fees to prioritize their transaction confirmation.

### 6. Why use integers rather than floating-point numbers for bitcoin amounts?

Floating-point arithmetic is inexact due to rounding errors. With money, even tiny rounding errors accumulate and cause discrepancies. Bitcoin uses 64-bit integers representing satoshis (the smallest unit, 1 BTC = 100,000,000 satoshis) to ensure exact calculations. This prevents lost or gained satoshis due to rounding.

### 7. Why does `total_input_value()` borrow `self`?

`total_input_value()` only needs to read the inputs, not modify them. By taking `&self` instead of `&mut self` or moving self, the function is non-destructive and can be called multiple times. The caller retains full ownership and can continue using the transaction. This is more flexible and efficient than moving or requiring mutable access.

### 8. Why does `add_input()` take `&mut self`?

`add_input()` needs to modify the transaction by pushing a new input into the vector. Therefore, it must take `&mut self` to express that it requires mutable access. This prevents accidental data races and makes it explicit that the method will change the transaction's state.

### 9. What happens when an input is moved into a transaction?

When an input is moved into the transaction via `add_input()`, ownership is transferred from the caller to the transaction. The caller no longer has access to that specific input value. The transaction now owns and manages the input's memory. This prevents the input from being used twice and ensures clear ownership semantics.

### 10. Why is `Result` preferable to `panic!` for validation failures?

`Result` allows callers to handle errors gracefully, decide how to recover, or propagate the error up the call stack. `panic!` terminates the entire program, preventing any recovery or custom error handling. By returning `Result`, the code is more robust, testable, and allows library code to integrate properly with diverse applications.

### 11. How do enums help model regular and coinbase inputs?

Enums allow the `InputKind` type to express that an input is either regular (spending a previous output) or coinbase (creating new coins from mining). Each variant carries the specific data it needs: regular inputs have a previous outpoint and sequence; coinbase inputs have a block height and reward. Pattern matching forces the code to handle both cases explicitly, preventing bugs from forgetting a variant.

### 12. How does the `BitcoinValue` trait reduce duplication?

Without the trait, we'd need separate `value()` methods on `TxOutput` and `InputKind`, and separate code to sum each type's values. The `BitcoinValue` trait defines a single interface that both types implement. Functions can now work generically over any type implementing `BitcoinValue`, and the default `value_in_btc()` method is defined once and reused by all implementers.

### Part 7: Ownership Experiment

The borrowed references pattern in Part 7 avoids unnecessary cloning:

**If we had cloned instead (inefficient):**
```rust
pub fn find_outputs_for_recipient(transaction: &Transaction, recipient: &str) -> Vec<TxOutput> {
    transaction
        .outputs
        .iter()
        .filter(|output| output.recipient == recipient)
        .map(|output| output.clone())  // CLONE - unnecessary copy!
        .collect()
}
```

**Using borrowed references (correct):**
```rust
pub fn find_outputs_for_recipient<'a>(
    transaction: &'a Transaction,
    recipient: &str,
) -> Vec<&'a TxOutput> {
    transaction
        .outputs
        .iter()
        .filter(|output| output.recipient == recipient)
        .collect()  // No clone needed - just references!
}
```

The lifetime annotation `'a` ensures that the returned references don't outlive the original transaction. This is the correct pattern: borrowing instead of cloning makes the code more efficient and expresses the relationship between data lifetimes.

## Design notes

### Part 9: UTXO Selection Algorithm

I implemented a simple "first-fit" algorithm that selects UTXOs in input order until the target is reached. This is straightforward and deterministic, making it easy to understand and test.

**Trade-offs:**
- **Pro:** Simple, deterministic, no allocation of unselected UTXOs
- **Con:** May create large change outputs if early UTXOs are large; doesn't minimize transaction size

A better algorithm would be "Coin Selection" or "Knapsack" solving, which minimizes either transaction size or the change output, reducing fees in the long run. However, the basic algorithm is sufficient for the assignment.

### Implementation Choices

1. **Validation order:** Validation checks are ordered by logical dependency and error likelihood (most obvious errors first).
2. **Error messages:** Each error message is specific and actionable, helping users understand what went wrong.
3. **Borrowing everywhere:** Part 7 functions use borrowed references and lifetime annotations to avoid unnecessary clones, demonstrating Rust's power for memory-safe, efficient code.
4. **Display formatting:** Transaction summary shows version, locktime, counts, totals, and fee in a human-readable format, handling invalid fees gracefully.

## Example output

```
Bitcoin Transaction Summary
==============================
Version: 2
Locktime: 0
Inputs: 2 (total: 120000 sats)
Outputs: 2 (total: 118000 sats)
Fee: 2000 sats
```

## Test Results

All 12 tests pass:
- ✅ 10 transaction tests (validation, totals, borrowing, error cases)
- ✅ 2 UTXO selection tests (sufficient funds, insufficient funds)

```bash
$ cargo test
running 10 tests in tests/transaction.rs
test valid_regular_transaction_passes_validation ... ok
test outputs_cannot_exceed_inputs ... ok
test no_inputs_is_invalid ... ok
test no_outputs_is_invalid ... ok
test valid_coinbase_transaction ... ok
test coinbase_mixed_with_regular_is_invalid ... ok
test multiple_coinbase_inputs_invalid ... ok
test highest_value_output_works ... ok
test find_outputs_for_recipient_works ... ok
test empty_txid_in_regular_input_is_invalid ... ok

running 2 tests in tests/utxo.rs
test selection_borrows_enough_utxos_in_slice_order ... ok
test insufficient_funds_is_an_error ... ok
```

## Submission Checklist

- ✅ All 10 parts implemented (including Part 8 payment example)
- ✅ No `#[ignore]` tests remaining
- ✅ 12 meaningful tests passing (exceeds 8 minimum)
- ✅ `cargo fmt --check` passes
- ✅ `cargo clippy --all-targets --all-features -- -D warnings` passes
- ✅ README.md complete with written answers and example output
- ✅ No external Bitcoin libraries used (pure Rust implementation)
