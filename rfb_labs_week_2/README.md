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

 - Explain why `InputKind` is an
  enum and how matching forces both regular and coinbase inputs to be handled.
 
```bash
InputKind is implemented as an enum to enforce the domain rule that Bitcoin inputs are strictly either a spent previous output (Regular) or a freshly minted block reward (Coinbase). Structuring them as separate variants prevents invalid data states—such as a Coinbase input possessing an OutPoint—directly at compile time, eliminating the need for fragile runtime checks across your codebase.  
 
Because Rust's match expressions are strictly exhaustive, the compiler forces developers to explicitly handle both Regular and Coinbase variants whenever processing an input; forgetting to handle either variant results in a compile error, guaranteeing that logic like value extraction or transaction fee calculations can never silently ignore a input type.
```

1. What is a Bitcoin transaction input?
- A transaction input is a reference to a specific Unspent Transaction Output (UTXO) created in a prior transaction. It spending-unlocks those satoshis and consumes them as the value source for the new transaction.
2. What is a Bitcoin transaction output?
- A transaction output defines a new UTXO created by the transaction. It specifies an amount in satoshis and an encumbrance (a script/pubkey locking script, such as P2WPKH) that dictates who can spend it in a future transaction input.
3. What is a UTXO?
- A UTXO (Unspent Transaction Output) is a discrete, atomic chunk of Bitcoin value that has been created by a transaction output but has not yet been consumed as an input by a subsequent valid transaction.
4. What does an outpoint identify?
- An outpoint uniquely identifies a specific UTXO on the blockchain using a tuple of (txid, vout)—where txid is the 32-byte hash of the producing transaction and vout (vector index) is the zero-based index of the output array within that transaction.
5. How is a transaction fee calculated?
- The transaction fee is calculated implicitly as the difference between the sum of all input satoshi values and the sum of all output satoshi values:$$\text{Fee} = \sum \text{Input Values} - \sum \text{Output Values}$$Because Bitcoin has no explicit "fee field," any unallocated input satoshis are claimed by the mining node that includes the block.
6. Why use integers rather than floating-point numbers for bitcoin amounts?
- Floating-point numbers (f32/f64) use IEEE 754 representation, which suffers from binary representation inaccuracy and rounding errors (e.g., $0.1 + 0.2 \neq 0.3$). Bitcoin consensus requires exact, deterministic arithmetic down to the smallest atomic unit (1 satoshi = $10^{-8}$ BTC), which is cleanly represented using unsigned 64-bit integers (u64).
7. Why does `total_input_value()` borrow `self`?
- It takes &self because it only needs to inspect and accumulate the value fields of the inputs without mutating the transaction or taking ownership of it. Taking self by value would consume the transaction, making it unusable after calculating the total.
8. Why does `add_input()` take `&mut self`?
- It requires &mut self (an exclusive mutable reference) because it modifies the internal state of the Transaction by pushing a new element into its inputs: Vec<InputKind> vector.
9. What happens when an input is moved into a transaction?
- Ownership of the InputKind value is transferred (moved) from the caller into the Transaction struct's inputs vector. The caller can no longer access or modify that input instance unless it is accessed via a reference through the Transaction.
10. Why is `Result` preferable to `panic!` for validation failures?
- panic! unrecoverably unwinds or aborts the current thread, which is unsafe for critical systems like transaction processing. Returning Result<(), TransactionError> allows callers to gracefully handle invalid data (e.g., rejecting an invalid transaction, notifying a user, or trying alternative coin selection) without crashing the runtime.
11. How do enums help model regular and coinbase inputs?
- An enum (InputKind) represents algebraic data types (sum types) where a variant can hold distinct structure definitions. A Regular variant safely encapsulates an OutPoint, value, and sequence, while a Coinbase variant encapsulates height and value. This guarantees type safety at compile time and prevents invalid field combinations.
12. How does the `BitcoinValue` trait reduce duplication?
- The BitcoinValue trait defines a shared interface (such as a .value() method) across different types like InputKind, TxOutput, and Utxo. This enables writing generic functions or iterator abstractions over any type containing satoshi amounts without duplicating summation or balance-checking logic.


**compiler error**
- This compiler error occurred when trying to pass owned values or call .into_iter() over self.outputs inside a method that takes &self (a shared reference). Because TxOutput contains a heap-allocated String (recipient), it does not implement the Copy trait. Attempting to take ownership of elements out of self.outputs through a shared borrow violates Rust's ownership and aliasing rules. The fix was to borrow elements using .iter(), which produces references (&TxOutput) instead of taking ownership.
```bash
error[E0308]: mismatched types                                                              
   --> src\transaction.rs:199:5
    |
190 | pub fn highest_value_output(transaction: &Transaction) -> Option<&TxOutput> {
    |                                                           ----------------- expected `Option<&TxOutput>` because of return type
...
199 |     transaction.outputs.into_iter().max_by_key(|out| out.value) 
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Option<&TxOutput>`, found `Option<TxOutput>`
    |
    = note: expected enum `Option<&TxOutput>`
               found enum `Option<TxOutput>`
help: try using `.as_ref()` to convert `Option<TxOutput>` to `Option<&TxOutput>`
    |
199 |     transaction.outputs.into_iter().max_by_key(|out| out.value).as_ref() 
    |                                                                +++++++++
```


## Design notes

Describe any choices you made, including your UTXO-selection trade-offs and (if
attempted) the optional transaction-state extension.
**UTXO Selection Algorithm & Trade-offs**
- A greedy linear-search selection strategy was implemented for UTXO selection. The function iterates through the available UTXO slice sequentially, accumulating funds until the cumulative balance meets or exceeds the required target.Trade-offs & Rationale:
- Simplicity & Predictability: O(N). Runs in linear time with minimal memory allocation. This makes the logic straightforward to reason about and deterministic for testing.
`Privacy & Fee Efficiency Trade-offs`
- Because UTXOs are selected in the order provided, this approach does not optimize for exact target matching like Branch and Bound, or for privacy like Single Random Draw. This can result in selecting more UTXOs than necessary, increasing the virtual size ( vB) of the transaction and therefore increasing mining fees.

**Lifetimes & Zero-Copy References**
- To enforce efficient memory usage, functions such as find_outputs_for_recipient and highest_value_output return borrowed references (&TxOutput) instead of cloning heap allocations like String recipients.Implementation Details:
`Explicit Lifetimes`
- Explicit lifetime annotations (e.g., Vec<&'a TxOutput>) tie the validity of returned references directly to the lifecycle of the parent Transaction reference (&'a Transaction).
`Functional Composition`
- Standard Rust iterator methods like .filter() and .max_by_key() enable clean, zero-copy borrowing without unsafe code or manual indexing.

**Type-State Pattern for Transaction Lifecycle**
- The transaction lifecycle — Unvalidated → Validated → Signed → Broadcast → Confirmed — was modeled using Rust’s Type-State Pattern with generic state wrappers `(TxState<S>)` and Zero-Sized Types (ZSTs).Key Properties:
- `Compile-Time Guarantees`
- State transitions consume self by value and return `TxState<NextState>`. This makes invalid operations, such as broadcasting an unvalidated transaction or re-signing a confirmed transaction, a compile-time error instead of a runtime panic.
- `Zero Overhead`
- The marker structs for each state contain no data. Therefore the state wrapper incurs zero runtime memory or CPU cost.

## Example output

Paste the output of `cargo run` here once Part 8 is complete.
```bash
--- Part 8 Transaction ---
Transaction (Version: 2, Locktime: 0)
Inputs (2):
  [0] Input(OutPoint: 0000000000000000000000000000000000000000000000000000000000000001:0, Value: 70000 sats, Seq: 4294967295)
  [1] Input(OutPoint: 0000000000000000000000000000000000000000000000000000000000000002:1, Value: 50000 sats, Seq: 4294967295)
Outputs (2):
  [0] Output(Value: 90000 sats, Recipient: bc1qreceiver, Type: P2wpkh)
  [1] Output(Value: 28000 sats, Recipient: bc1qsender, Type: P2wpkh)
Total In: 120000 sats | Total Out: 118000 sats
Fee: 2000 sats

--- Part 10 State Machine ---
State: Validated
State: Signed (Sig: 30440220...sig)
State: Broadcast (TXID: txid_abc123)
State: Confirmed at block 840000

```


