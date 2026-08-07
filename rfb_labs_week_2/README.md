# Rust for Bitcoin 2.0 — Week 2

Build a simplified Bitcoin transaction model while practising structs, enums, traits, ownership, borrowing, collections, and `Result`-based error handling.

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

---

## Written answers

### Part 7 Ownership Compiler Error Observation

When attempting to implement `highest_value_output` by taking ownership or calling `.into_iter()` directly on `transaction.outputs` through a shared reference (`&Transaction`), the Rust compiler rejects the code with the following error:

```text
error[E0507]: cannot move out of `transaction.outputs` which is behind a shared reference
   --> src/transaction.rs:113:5
    |
113 |     transaction.outputs.into_iter().max_by_key(|o| o.value)
    |     ^^^^^^^^^^^^^^^^^^^ ----------- `transaction.outputs` moved due to this method call
    |     |
    |     help: consider calling `.iter()` instead of `.into_iter()`
```

#### Explanation of Root Cause:
In Rust's ownership model, a shared reference `&Transaction` grants read-only borrowing privileges to inspect the transaction data without taking ownership or destroying the original value. Calling `.into_iter()` on `transaction.outputs` attempts to consume (move) the `Vec<TxOutput>` elements out of the struct. Because the function only holds a shared reference (`&Transaction`), moving elements out of it would leave the underlying `Transaction` struct in an incomplete, invalidated state for all other potential readers. 

To fix this issue without copying memory or violating ownership rules, we call `.iter()` instead of `.into_iter()`. This produces an iterator yielding shared references `&TxOutput` that borrow from the `Transaction` struct, allowing us to safely return `Option<&TxOutput>`.

---

### Questions & Answers

#### 1. What is a Bitcoin transaction input?
A Bitcoin transaction input is a pointer to a specific unspent transaction output (UTXO) created in a prior transaction that is being consumed (spent) by the current transaction. In a regular transaction input, it contains an `OutPoint` (the parent transaction ID `txid` and index `vout`), the value being spent, a `sequence` number (used for locktimes and Replace-By-Fee), and cryptographic unlocking scripts or signatures (`scriptSig` / `witness`). In a coinbase transaction input, it represents a special block reward input created by miners containing the block height and arbitrary miner data (coinbase text).

#### 2. What is a Bitcoin transaction output?
A Bitcoin transaction output is a discrete coin created by a transaction that specifies an amount of bitcoin (denominated in satoshis) and the locking conditions required to spend it in the future. In our model, each `TxOutput` consists of a `value` (in satoshis), a `recipient` address identifier, and an `output_type` (such as `P2pkh`, `P2wpkh`, `P2tr`, or `OpReturn` data payloads).

#### 3. What is a UTXO?
A UTXO (Unspent Transaction Output) is an immutable output created by a previous transaction that has not yet been spent as an input by any valid block on the main blockchain. The full global collection of UTXOs forms the Bitcoin UTXO set. Bitcoin does not use account balances; instead, a user's total spendable balance is computed by scanning and summing the values of all UTXOs that match addresses or public keys controlled by the user's wallet.

#### 4. What does an outpoint identify?
An outpoint (`OutPoint`) uniquely identifies a specific UTXO across the global Bitcoin blockchain network. It consists of a tuple containing a 32-byte transaction hash (`txid`) and a zero-based output index (`vout`). The `txid` specifies the exact transaction that produced the output, while `vout` indicates which output position within that transaction's output array is being referenced.

#### 5. How is a transaction fee calculated?
In Bitcoin, the transaction fee is not stored as an explicit output inside the transaction. Instead, it is implicitly calculated as the unassigned residual value:
$$\text{Fee} = \sum \text{Input Values} - \sum \text{Output Values}$$
The miner who successfully mines the block containing the transaction includes this implicit residual difference in their coinbase transaction reward. If the total output value exceeds total input value, the transaction is invalid.

#### 6. Why use integers rather than floating-point numbers for bitcoin amounts?
Financial software handling cryptocurrency transactions must avoid floating-point representation errors inherent to IEEE-754 standards (such as binary rounding inaccuracies like `0.1 + 0.2 = 0.30000000000000004`). In Bitcoin, 1 BTC equals 100,000,000 satoshis (`1 sat = 1e-8 BTC`). Denominating all amounts as integer satoshis (`u64`) guarantees exact, lossless arithmetic, determinism across different CPU architectures, and precise value conservation auditing.

#### 7. Why does `total_input_value()` borrow `self`?
`total_input_value(&self)` takes a shared immutable reference `&self` because calculating the total input value only requires inspecting (reading) the values stored inside the transaction's `inputs` vector. Borrowing `self` immutably allows caller functions to query the transaction value without taking ownership (which would destroy the transaction) and without requiring exclusive mutable access.

#### 8. Why does `add_input()` take `&mut self`?
`add_input(&mut self, input: InputKind)` requires a unique mutable reference `&mut self` because it modifies the internal state of the `Transaction` struct by pushing a new `InputKind` element into `self.inputs`. Under Rust's aliasing and mutability rules, mutating state requires exclusive mutable access to ensure no concurrent reads or writes introduce data races.

#### 9. What happens when an input is moved into a transaction?
When an `InputKind` value is passed into `add_input(mut self, input: InputKind)`, ownership of that `input` data is transferred (moved) into the `Transaction` struct and stored within its `inputs` `Vec`. The caller function surrenders ownership of the `input` variable, rendering it unaccessible in the caller's scope, while the `Transaction` assumes responsibility for managing the allocated memory.

#### 10. Why is `Result` preferable to `panic!` for validation failures?
In systems programming and financial applications, invalid user input or mismatched transaction parameters are expected runtime operational conditions rather than fatal software bugs. Using `Result<(), TransactionError>` allows functions to report validation failures gracefully to callers, enabling recovery, user feedback, or alternative logic. Calling `panic!` would abruptly crash the entire thread/process, leading to potential denial-of-service vulnerability and unrecoverable application state.

#### 11. How do enums help model regular and coinbase inputs?
`InputKind` is defined as an enum with two distinct variants: `Regular` (containing `previous_output`, `value`, and `sequence`) and `Coinbase` (containing `block_height` and `reward`). Enums allow Rust to represent data that can take one of several distinct variants safely within a single type. When processing `InputKind` values, Rust's pattern matching (`match`) forces the developer to handle both variants explicitly, preventing runtime errors or missing logic for coinbase block rewards versus standard UTXO spends.

#### 12. How does the `BitcoinValue` trait reduce duplication?
The `BitcoinValue` trait defines a shared interface (`value(&self) -> u64`) and provides a default blanket method implementation (`value_in_btc(&self) -> f64`). By implementing `BitcoinValue` for `TxOutput` and `InputKind`, both types gain standardized access to satoshi values and automated BTC conversions without duplicating formula logic across separate structs.

---

## Design notes

### UTXO Selection Trade-offs:
1. **Slice-Order Selection (FIFO / Direct Accumulation)**:
   - *Pros*: Simple, predictable $O(N)$ computational complexity, low execution overhead.
   - *Cons*: Can accumulate multiple small UTXOs unnecessarily, increasing transaction virtual size (`vsize`) and fee costs.
2. **Knapsack / Branch-and-Bound Coin Selection**:
   - *Pros*: Aims for exact target matching without creating change outputs (saving output creation fees and reducing UTXO set bloat).
   - *Cons*: Higher algorithmic complexity ($O(2^N)$ worst-case), requiring fallback algorithms when an exact match is unobtainable.
3. **Privacy Considerations (Common Input Ownership Heuristic)**:
   - Combining multiple UTXOs in a single transaction reveals to blockchain analytics that all consumed inputs belong to the same entity. Selecting a single large UTXO when available preserves wallet privacy better than merging multiple small outputs.

### Optional Transaction State Extension:
- In production Bitcoin nodes, transactions progress through formal state transitions:
  `Created` $\rightarrow$ `Validated` $\rightarrow$ `Signed` $\rightarrow$ `Broadcast` $\rightarrow$ `Confirmed` (or `Rejected`).
- State machine typestate pattern in Rust can enforce at compile time that an unvalidated transaction cannot be signed or broadcast, preventing invalid state transitions.

---

## Example output

```text
=== Bitcoin Transaction Summary ===
Version: 2
Locktime: 0
Inputs (2):
  [0] Regular Input [1111111111111111111111111111111111111111111111111111111111111111:0] (70000 sats / 0.00070000 BTC, sequence: 4294967295)
  [1] Regular Input [2222222222222222222222222222222222222222222222222222222222222222:1] (50000 sats / 0.00050000 BTC, sequence: 4294967295)
Outputs (2):
  [0] 90000 sats (0.00090000 BTC) -> bc1qreceiver [P2wpkh]
  [1] 28000 sats (0.00028000 BTC) -> bc1qsender [P2wpkh]
Total Input Value:  120000 sats (0.00120000 BTC)
Total Output Value: 118000 sats (0.00118000 BTC)
Calculated Fee:     2000 sats (0.00002000 BTC)
```
