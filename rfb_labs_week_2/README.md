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

### Ownership Compiler Error from Part 7

When attempting to use a value after it has been moved into a transaction:

```
error[E0382]: borrow of moved value: `input`
  --> examples/ownership_experiment.rs:19:53
   |
 6 |     let input = InputKind::Regular {
   |         ----- move occurs because `input` has type `InputKind`, which does not implement the `Copy` trait
...
16 |     transaction.add_input(input);
   |                           ----- value moved here
...
19 |     println!("Attempting to use moved value: {:?}", input);
   |                                                     ^^^^^ value borrowed here after move
```

**What caused it:**

This error occurs because `add_input()` takes ownership of the `InputKind` value by accepting it as a parameter without a reference (`input: InputKind` not `input: &InputKind`). When we call `transaction.add_input(input)`, the ownership of `input` is transferred (moved) into the transaction's internal `Vec<InputKind>`. After this move, the original `input` variable is no longer valid in the calling scope.

When we try to use `input` again (e.g., in a `println!`), Rust's borrow checker prevents this because the value has already been moved. The `InputKind` enum doesn't implement the `Copy` trait, so it follows move semantics rather than copy semantics. This is intentional in our design - once an input is added to a transaction, we don't want multiple owners of the same input, as that could lead to double-spending scenarios or inconsistent state.

---

1. What is a Bitcoin transaction input?

A Bitcoin transaction input is a reference to a previous transaction's output that is being spent. It is a reference to a previous output from an earlier transaction, the value being spent, and additional data like sequence number. Regular inputs reference UTXOs, while coinbase inputs (used in block rewards) reference the block height and mining reward.

2. What is a Bitcoin transaction output?

A Bitcoin transaction output specifies an amount of bitcoin (in satoshis) and the recipient's address. It defines where the funds are going and how much. Each output has a value, a recipient address, and an output type (P2PKH, P2WPKH, P2TR, or OpReturn for data storage).

3. What is a UTXO?

UTXO stands for Unspent Transaction Output. It's a transaction output that hasn't been spent yet and is available to be used as an input in a future transaction. UTXOs represent the spendable bitcoin balance in the Bitcoin network - your wallet balance is the sum of all UTXOs you control.

4. What does an outpoint identify?

An outpoint uniquely identifies a specific output within a specific transaction. It consists of a transaction ID (txid) and an output index (vout). It actually represents the index of outputs chosen from for a selected input

5. How is a transaction fee calculated?

The transaction fee is calculated as the difference between the total value of all inputs and the total value of all outputs: `fee = total_inputs - total_outputs`. The miner who includes the transaction in a block claims this fee as a reward. If outputs exceed inputs, the transaction is invalid.

6. Why use integers rather than floating-point numbers for bitcoin amounts?

Integers ensure precise, deterministic calculations without rounding errors. Floating-point arithmetic can introduce small errors due to binary representation limitations, which is unacceptable in financial systems where exact values are critical. Using satoshis (the smallest unit, 1 BTC = 100,000,000 sats) as u64 integers guarantees perfect accuracy and consistency across all implementations.

7. Why does `total_input_value()` borrow `self`?

It borrows `self` with `&self` because it only needs to read the input values, not modify the transaction. Borrowing allows multiple simultaneous reads and avoids taking ownership, which would consume the transaction and make it unusable after the call. This follows Rust's principle of using the least privilege necessary.

8. Why does `add_input()` take `&mut self`?

It takes `&mut self` because it modifies the transaction by adding an input to the internal vector. The mutable reference allows the function to change the transaction's state while maintaining exclusive access during the modification, preventing data races. After the call, the transaction remains owned by the caller.

9. What happens when an input is moved into a transaction?

When an input is moved into a transaction ownership transfers from the caller to the transaction's internal `Vec<InputKind>`. The input has now been moved and previous variable holding the input becomes invalid

10. Why is `Result` preferable to `panic!` for validation failures?
panic! allows the programme to crash entirely and the errors are unrecoverable.It does not send a clear message to the user as to how to handle the error. Whether to try again or something is missing. Whereas`Result` allows expected failures to be handled gracefully in code, giving the caller control over error recovery. It provides a more nuanced approach to handling errors and provides the user a clear message as to what is missing or whether to try again, for example in accessing a file maybe returns "file not found". Validation failures (like outputs exceeding inputs) are expected conditions, custom errors. Using `Result` makes errors explicit in the type signature, allows programs to recover or provide user-friendly error messages rather than crashing.

11. How do enums help model regular and coinbase inputs?

Enums allow us to represent two fundamentally different types of inputs in a single type. Regular inputs reference previous UTXOs (with outpoint, value, sequence), while coinbase inputs represent newly minted coins (with block height and reward). The enum forces exhaustive pattern matching, ensuring both variants are always handled correctly and preventing invalid states like mixing both types.

12. How does the `BitcoinValue` trait reduce duplication?

The `BitcoinValue` trait provides a common interface for accessing values across different types (`TxOutput`, `InputKind`). Instead of writing separate logic for outputs and each input variant, we can write generic code that works with any type implementing the trait. This reduces code duplication in functions that need to sum values or perform calculations regardless of the specific type.

## Design notes

Describe any choices you made, including your UTXO-selection trade-offs and (if
attempted) the optional transaction-state extension.

### UTXO Selection Algorithm

The implemented `select_utxos()` function it iterates through available UTXOs in the order provided and accumulates them until the target amount is reached or exceeded.

**Better Alternatives:**

1. **Branch and Bound**: Finds the optimal set of UTXOs that minimizes the number of inputs while meeting the target. More complex but produces smaller, cheaper transactions.

2. **Largest-First**: Selects the largest UTXOs first, useful for consolidating small UTXOs and reducing fragmentation.

3. **Smallest-First**: Selects the smallest UTXOs first, helps spend dust and reduce overall UTXO set size.

4. **Privacy-Aware Selection**: Avoids creating round-number changes or linking UTXOs from different sources to improve transaction privacy.


### Transaction State (Part 10 - Not Implemented)

The optional transaction state machine (Created → Validated → Signed → Broadcast → Confirmed/Rejected) was not implemented. If implemented, it would use Rust's typestate pattern with zero-sized marker types to enforce valid state transitions at compile time, preventing operations like broadcasting an unvalidated transaction.

## Example output

Paste the output of `cargo run` here once Part 8 is complete.
     Running `target/debug/rfb-labs-week-2`
Transaction:
  Version: 2
  Locktime: 0
  Inputs: 2
  Outputs: 2
  Total input: 120000 sats
  Total output: 118000 sats
  Fee: 2000 sats