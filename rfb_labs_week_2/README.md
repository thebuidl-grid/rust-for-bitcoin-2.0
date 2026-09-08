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

   A bitcoin transaction input holds a reference to a UTXO in a senders wallet. A transaction can hold multiple inputs to be able to provide enough value for the recipient's payment, and transaction fees.

2. What is a Bitcoin transaction output?

   A Bitcoin transaction output is created by a transaction. It specifies an amount of bitcoin and a locking condition controlling how that amount can later be spent. A spendable transaction output is called a UTXO while it remains unspent.

3. What is a UTXO?

   UTXO means “unspent transaction output.” It represents an output that has not yet been spent and can be referenced by an input in a later transaction.

4. What does an outpoint identify?

   A transaction outpoint is used to reference a UTXO that will be used in a Bitcoin transaction. It contains the transaction ID (`txid`) and the `vout`, which is the output index. Because a single transaction can create multiple UTXOs with the same transaction ID, the `vout` is used to distinguish between them.

5. How is a transaction fee calculated?

   A Bitcoin transaction fee is calculated by subtracting the total value of all transaction outputs from the total value of all transaction inputs. The outputs include the receiver’s payment and any change returned to the sender. Therefore, the formula is `fee = total inputs − (receiver payment + change)`.

6. Why use integers rather than floating-point numbers for bitcoin amounts?

   Integers represent exact values and avoid the rounding errors that can occur when monetary amounts are stored using floating-point numbers. Therefore, we store Bitcoin amounts in satoshis—the smallest unit of bitcoin—instead of using decimal bitcoin values.

7. Why does `total_input_value()` borrow `self`?

   `self` is borrowed because we only need to read values from the transaction. If the method took `self` directly, ownership of the transaction would be moved whenever `transaction.total_input_value()` was called, making the transaction unavailable to the caller afterward.

8. Why does `add_input()` take `&mut self`?

   `add_input()` takes `&mut self` because it modifies the transaction by adding an input to its `inputs` vector. A mutable reference allows the method to change the transaction without taking ownership of it. Without `mut`, Rust would not permit the method to modify the transaction.

9. What happens when an input is moved into a transaction?

   When an input is moved into a transaction, ownership of the input is transferred to the transaction’s `inputs` vector. The original variable can no longer be used by the caller unless the value is returned or explicitly cloned. The input remains valid for as long as the transaction owns it.

10. Why is `Result` preferable to `panic!` for validation failures?

    `Result` is preferable because it allows validation errors to be handled or propagated without crashing the program. Invalid transaction data is an expected failure that callers may recover from, while `panic!` immediately stops normal program execution.

11. How do enums help model regular and coinbase inputs?

    The `InputKind` enum uses struct-like variants to model `Regular` and `Coinbase` inputs. An input can be one variant at a time, so it cannot be both regular and coinbase simultaneously. Each variant can store the fields relevant to that input type, and pattern matching ensures that both variants are handled explicitly.

12. How does the `BitcoinValue` trait reduce duplication?

    The `BitcoinValue` trait defines a common `value()` method that outputs and input variants must implement. It also provides the shared `value_in_btc()` calculation once, so each implementing type does not need to duplicate the conversion from satoshis to bitcoin. This allows different types to follow the same interface while keeping their own value-extraction logic.

## Ownership and borrowing experiment

I initially iterated over the transaction outputs by value:

```rust
for txn in transaction.outputs {
    if txn.recipient == recipient {
        recipients_transactions.push(txn);
    }
}
```

This produced:

```text
error[E0308]: mismatched types
expected struct `Vec<&TxOutput>`
   found struct `Vec<TxOutput>`
```

Iterating by value made each `txn` an owned `TxOutput`, so Rust inferred the
result as `Vec<TxOutput>`. The function was required to return references borrowed
from the transaction. I fixed it by iterating over `&transaction.outputs`, making
each item an `&TxOutput` and allowing the returned references to remain tied to
the transaction's lifetime.

## Design notes

I represented Bitcoin amounts as `u64` integers containing satoshis. This keeps monetary calculations exact and avoids the rounding errors associated with floating-point values. The simplified model assumes that the supplied amounts are within Bitcoin’s valid monetary range.

I used `Result` and specific `TransactionError` variants for transaction validation and UTXO-selection failures. This allows callers to handle or propagate expected errors without crashing the program. The implementation covers the validation failures required by the assignment, although a production transaction model would likely include additional validation and error cases.

The UTXO-selection algorithm selects coins in their existing input order until their combined value meets or exceeds the target. I used a direct for loop to keep the implementation simple and readable. In a larger production implementation, I might use .iter() with operations such as filtering and searching to first find an exact match or choose a combination that reduces the number of inputs and amount of change. However, I kept the simpler input-order strategy because it is predictable. Its trade-off is that it may select more UTXOs than necessary and produce unnecessary change.

## Example output

Paste the output of `cargo run` here once Part 8 is complete.

```text
Transaction(version=2, locktime=0, inputs=2, outputs=2, total input=120000 sats, total output=118000 sats, fee=2000 sats)
```
