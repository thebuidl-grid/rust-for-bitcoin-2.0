# Rust for Bitcoin 2.0 — Week 2

Build a simplified Bitcoin transaction model while practising structs, enums,
traits, ownership, borrowing, collections, and `Result`-based error handling.

## Written answers

### 1. What is a Bitcoin transaction input?
A Bitcoin transaction input (`vin`) is a reference to a specific previously unspent transaction output (UTXO) being spent, identified by its `OutPoint` (`txid:vout`). It includes unlocking data (such as cryptographic signatures, scriptWitness, or scriptSig) proving authorization to spend those funds under the locking script conditions defined in the referenced UTXO.

### 2. What is a Bitcoin transaction output?
A Bitcoin transaction output (`vout`) defines a new spendable coin created by a transaction. It specifies a monetary value in satoshis ($1\text{ BTC} = 100,000,000\text{ sats}$) and a locking script (`scriptPubKey` or address type such as P2PKH, P2WPKH, P2TR, or OP_RETURN) that encumbers the funds and dictates the conditions required to spend them in the future.

### 3. What is a UTXO?
A UTXO (Unspent Transaction Output) is an output from a past confirmed or mempool transaction that currently exists in the global UTXO set database. UTXOs are discrete, indivisible units of bitcoin value that can only be spent in their entirety as inputs to a new transaction.

### 4. What does an outpoint identify?
An outpoint (`OutPoint`) is a unique 2-tuple coordinate (`txid`, `vout`) that globally pinpoints a specific transaction output. `txid` is the 32-byte hex hash of the origin transaction, and `vout` is the 0-based integer index of the specific output within that transaction's output vector.

### 5. How is a transaction fee calculated?
The transaction fee is calculated implicitly as the unallocated difference between the total input value and the total output value:
$$\text{Fee} = \sum \text{Input Values} - \sum \text{Output Values}$$
Any input satoshis not explicitly assigned to a recipient or change output are collected as transaction fees by the miner who mines the block.

### 6. Why use integers rather than floating-point numbers for bitcoin amounts?
Floating-point numbers (IEEE 754 `f32`/`f64`) introduce binary representation inaccuracies, rounding errors, and non-deterministic CPU arithmetic behaviors (e.g., `0.1 + 0.2 != 0.3`). In a financial consensus network, monetary calculations must be strictly exact and deterministic across all operating systems and CPU architectures. Representing values as integer satoshis guarantees exact, non-fractional accounting.

### 7. Why does `total_input_value()` borrow `self`?
`total_input_value(&self)` only reads the values stored in `self.inputs` to compute a scalar sum (`u64`). Borrowing `self` immutably (`&self`) allows callers to query the total input value without destroying or consuming the `Transaction` instance, enabling subsequent method calls on the same transaction object.

### 8. Why does `add_input()` take `&mut self`?
`add_input(&mut self, input: InputKind)` mutates the internal state of the `Transaction` struct by appending `input` to its `inputs: Vec<InputKind>` vector. Mutating a struct field requires an exclusive mutable reference (`&mut self`) in Rust to enforce thread safety and prevent data races.

### 9. What happens when an input is moved into a transaction?
When an `InputKind` value is passed by value to `add_input(input)`, ownership of that memory is transferred (moved) from the caller's scope into the `Transaction`'s internal `self.inputs` vector. The caller can no longer access or reuse the original variable after the move, preventing double-use or dangling references.

### 10. Why is `Result` preferable to `panic!` for validation failures?
A `panic!` abruptly halts process execution and crashes the node or thread. Validation failures (such as invalid outputs or insufficient funds) are expected runtime occurrences when processing untrusted network data. Returning `Result<T, TransactionError>` allows callers to handle invalid input gracefully, log diagnostic information, and remain operational.

### 11. How do enums help model regular and coinbase inputs?
Regular inputs and coinbase inputs have fundamentally different structures: regular inputs reference a previous `OutPoint` and `sequence`, whereas coinbase inputs carry `block_height` and lack a previous UTXO outpoint. An `enum` (`InputKind`) enables type-safe modeling of these distinct variants within a single collection (`Vec<InputKind>`), forcing pattern matching (`match`) at compile time so that all code paths explicitly handle both input types.

### 12. How does the `BitcoinValue` trait reduce duplication?
The `BitcoinValue` trait defines a shared interface (`value(&self) -> u64` and `value_in_btc(&self) -> f64`) across different types (`TxOutput`, `InputKind`). This allows polymorphic handling of monetary amounts without duplicating conversion logic.

---

## Part 7 — Ownership Compiler Error

Attempting to move `transaction.outputs` out of a shared reference `&Transaction`:

```text
error[E0507]: cannot move out of `transaction.outputs` which is behind a shared reference
   --> src\transaction.rs:159:19
    |
159 |     let outputs = transaction.outputs;
    |                   ^^^^^^^^^^^^^^^^^^^ move occurs because `transaction.outputs` has type `Vec<TxOutput>`, which does not implement the `Copy` trait
```

### Explanation of Cause
The function `highest_value_output(transaction: &Transaction)` receives a shared (immutable) reference `&Transaction`. In Rust, you cannot move owned data (such as a `Vec<TxOutput>`) out of a borrowed reference because the caller still owns the `Transaction` struct and expects its fields to remain intact. To fix this error, we iterate over borrowed references to the outputs using `.iter()` rather than taking ownership.

---

## Design notes

1. **UTXO Selection Algorithm**: The basic `select_utxos` implementation iterates through the available UTXO slice in order, accumulating coins until the requested target is reached. While simple $O(N)$ and predictable, this approach can leave small residual change outputs ("dust") or select more inputs than necessary. In production, algorithms like **Branch and Bound (BnB)** search for exact matches to eliminate change outputs entirely (reducing transaction size and fees), while **Knapsack Solver** or **Largest-First** select optimal combinations to minimize input counts.
2. **Error Safety**: All validation routines in `Transaction::validate()` return descriptive `TransactionError` variants without invoking `panic!`.

---

## Example output

```text
Transaction v2 (locktime 0) [2 inputs (120000 sats), 2 outputs (118000 sats), fee: 2000 sats]
```
