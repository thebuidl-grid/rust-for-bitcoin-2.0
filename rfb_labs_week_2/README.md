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

### Ownership Compiler Error (Part 7)

```text
error[E0507]: cannot move out of `*o` which is behind a shared reference
   --> src/transaction.rs:148:47
    |
148 | ...on<TxOutput> = opt.map(|o| *o); // Fails here: cannot move out of sh...
    |                               ^^ move occurs because `*o` has type `TxOutput`, which does not implement the `Copy` trait
    |
note: if `TxOutput` implemented `Clone`, you could clone the value
   --> src/transaction.rs:14:1
    |
 14 | pub struct TxOutput {
    | ^^^^^^^^^^^^^^^^^^^ consider implementing `Clone` for this type
...
148 |     let owned: Option<TxOutput> = opt.map(|o| *o); // Fails here: canno...
    |                                               -- you could clone this value
```

**Explanation of the cause:**
This error occurs because we have a shared reference to the `Transaction` (`&Transaction`) and try to dereference a shared reference to a `TxOutput` (`*o` where `o: &TxOutput`) in order to move/return it by value (creating an owned `TxOutput`). In Rust, moving a value out of a borrowed collection is forbidden because it would leave the collection in an uninitialized or corrupted state, violating memory safety. Since `TxOutput` does not implement `Copy` or `Clone`, Rust cannot duplicate the value, forcing a compiler error. To solve this, we must return a borrowed reference `&TxOutput` instead of moving the value.

---

### Questions & Answers

1. **What is a Bitcoin transaction input?**
   A Bitcoin transaction input is a reference to an unspent output of a previous transaction (called a UTXO), paired with a witness/signature that unlocks those funds. It represents the source of funds for the transaction. In our model, regular inputs specify the outpoint and value being spent, while coinbase inputs represent newly minted block rewards.

2. **What is a Bitcoin transaction output?**
   A Bitcoin transaction output specifies the destination and amount of funds being sent. It contains a value (in satoshis) and an output type representing locking scripts (like P2PKH, P2WPKH, P2TR, or OP_RETURN data). This locking script specifies who can spend these funds next.

3. **What is a UTXO?**
   A UTXO (Unspent Transaction Output) is a transaction output that has not yet been spent as an input in another transaction. In Bitcoin, balances are not tracked as account balances, but as a set of individual UTXOs. To make a payment, a transaction consumes existing UTXOs and creates new UTXOs.

4. **What does an outpoint identify?**
   An outpoint (`OutPoint`) uniquely identifies a specific transaction output on the blockchain. It consists of the transaction hash (`txid`) that created the output, and the specific output index (`vout`) within that transaction.

5. **How is a transaction fee calculated?**
   A transaction fee is calculated implicitly as the difference between the total value of all inputs and the total value of all outputs:
   $$\text{Fee} = \sum(\text{Inputs}) - \sum(\text{Outputs})$$
   The remainder is kept by the miner who includes the transaction in a block.

6. **Why use integers rather than floating-point numbers for bitcoin amounts?**
   Floating-point numbers (like `f32` or `f64`) suffer from precision and rounding errors (IEEE 754 limitations) when representing decimals. In financial ledgers, even a microscopic rounding error is unacceptable and can lead to consensus failures across nodes. Using integer satoshis ($1 \text{ BTC} = 10^8 \text{ satoshis}$) ensures exact arithmetic and consensus consistency.

7. **Why does `total_input_value()` borrow `self`?**
   `total_input_value()` only needs to read the inputs to sum up their values; it does not need to modify the transaction or take ownership of it. Borrowing `self` immutably (`&self`) allows the transaction to be read multiple times without moving or mutating it.

8. **Why does `add_input()` take `&mut self`?**
   `add_input()` appends a new input to the transaction's internal vector, modifying the `Transaction` state. Modifying a structure requires a mutable reference (`&mut self`).

9. **What happens when an input is moved into a transaction?**
   When an input is moved into `add_input(self, input: InputKind)`, the ownership of that input is transferred to the transaction. The caller can no longer access or reuse that input variable, preventing double-use and ensuring that the transaction owns its data.

10. **Why is `Result` preferable to `panic!` for validation failures?**
    Transaction validation failures (e.g. invalid fee, mixed inputs) are standard runtime issues that programs must handle gracefully. Using `panic!` abruptly crashes the entire process, which is catastrophic for a node or wallet. Returning a `Result` allows the caller to catch the error, log it, notify the user, or reject the peer without crashing.

11. **How do enums help model regular and coinbase inputs?**
    Enums allow us to group distinct types of inputs (`Regular` and `Coinbase`) under a single unified type (`InputKind`) while keeping their data structures separate (since coinbase inputs don't have outpoints). Rust's pattern matching on enums guarantees that developers handle both variants explicitly, preventing bugs where coinbase rules are accidentally ignored.

12. **How does the `BitcoinValue` trait reduce duplication?**
    The `BitcoinValue` trait provides a common interface for anything that carries a value in satoshis. Both `TxOutput` and `InputKind` can implement it, allowing us to share common methods like `value_in_btc()` without duplicate code.

## Design notes

### UTXO-selection trade-offs
The implemented coin selection algorithm selects UTXOs in FIFO/slice order until the target is met.
- **Trade-offs:**
  - **Pros:** It is simple, deterministic, and runs in linear time $O(n)$.
  - **Cons:** It doesn't attempt to minimize the number of UTXOs (to reduce transaction size and fee), nor does it try to find an exact match to avoid creating a "change" output, which increases privacy leakage and consumes more blockchain space.
  - **Alternative (Bonus):** A better approach is the Branch-and-Bound (BnB) algorithm, which searches for an exact UTXO match to eliminate change outputs, or Single Random Draw (SRD) to avoid generating dust.

## Example output

```text
Transaction v2 (locktime: 0): 2 inputs, 2 outputs, total input: 120000 sats, total output: 118000 sats, fee: 2000 sats
```
