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

## Ownership and Borrowing Experiment

### Part 7 Compiler Error Output

```text
error[E0382]: borrow of moved value: `output`
  --> src/main.rs:13:34
   |
 7 |     let output = rfb_labs_week_2::TxOutput {
   |         ------ move occurs because `output` has type `TxOutput`, which does not implement the `Copy` trait
...
12 |     transaction.add_output(output);
   |                            ------ value moved here
13 |     println!("Output value: {}", output.value);
   |                                  ^^^^^^^^^^^^ value borrowed here after move
```

### Explanation

- **What value was moved:** The variable `output` of type `TxOutput` was moved when passed by value into `transaction.add_output(output)`.
- **Why Rust rejected later use:** `TxOutput` contains a `String` (`recipient`) and does not implement the `Copy` trait. When `output` was passed by value to `add_output(self, output: TxOutput)`, ownership was transferred to `transaction.outputs`. Consequently, `output` in `main()` became invalid/uninitialized, making any subsequent attempt to read or borrow `output.value` a compile-time error.
- **How borrowing changes the situation:** Borrowing (`&transaction` or `&output`) creates a reference pointing to the existing data without moving ownership. The underlying memory remains owned by its original container, enabling multiple components to inspect the data safely under Rust's borrowing rules.

## Written answers

1. **What is a Bitcoin transaction input?**  
   A Bitcoin transaction input unlocks and consumes an existing Unspent Transaction Output (UTXO) created in a prior transaction. It references the previous output via an `OutPoint` (TXID and output index `vout`), supplies the satoshi value being spent, and includes a sequence number for locktime/relative delay controls. In block creation, a special coinbase input introduces newly minted bitcoins and block fees without consuming a prior UTXO.

2. **What is a Bitcoin transaction output?**  
   A Bitcoin transaction output (`TxOutput`) specifies a monetary amount in satoshis and an encumbrance (locking script / recipient condition) that dictates who can spend those funds in a future transaction. Outputs can also include `OpReturn` data payloads which carry zero monetary value.

3. **What is a UTXO?**  
   A UTXO (Unspent Transaction Output) represents a discrete chunk of bitcoin that has been created as an output in a past transaction but has not yet been spent as an input in any valid transaction. The aggregate set of all UTXOs defines the current ledger state of Bitcoin.

4. **What does an outpoint identify?**  
   An outpoint (`OutPoint`) uniquely identifies a specific unspent output on the Bitcoin blockchain using a pair of identifiers: the 32-byte transaction hash (`txid`) in which the output was created, and a zero-based index (`vout`) denoting the output position within that transaction.

5. **How is a transaction fee calculated?**  
   In Bitcoin, transaction fees are implicit rather than explicitly listed as a separate field. The fee is calculated as the total satoshi value of all transaction inputs minus the total satoshi value of all transaction outputs (`fee = total_inputs - total_outputs`). Miners claim this surplus value when mining a block.

6. **Why use integers rather than floating-point numbers for bitcoin amounts?**  
   Floating-point arithmetic (`f32`/`f64`) introduces non-deterministic rounding errors and precision issues (e.g. `0.1 + 0.2 != 0.3`). In financial systems and consensus-critical protocols, exact precision is mandatory. Representing amounts as 64-bit unsigned integers (`u64`) measuring satoshis guarantees exact integer arithmetic without precision loss or rounding anomalies.

7. **Why does `total_input_value()` borrow `self`?**  
   `total_input_value()` only needs to read the satoshi values of the transaction's existing inputs. Taking `&self` allows callers to query the total input value multiple times without relinquishing or mutating ownership of the `Transaction`.

8. **Why does `add_input()` take `&mut self`?**  
   `add_input()` appends a new input to the `Transaction` struct's internal `inputs` vector. Because modifying the internal collection alters the struct's state, Rust requires a mutable reference (`&mut self`) to enforce exclusive access and prevent data races or concurrent modifications.

9. **What happens when an input is moved into a transaction?**  
   Passing an `InputKind` by value into `add_input(input)` transfers ownership of the `InputKind` value from the caller to the `Transaction` instance (which stores it in its `inputs: Vec<InputKind>`). The original variable in the caller's stack frame becomes uninitialized and can no longer be accessed unless re-assigned.

10. **Why is `Result` preferable to `panic!` for validation failures?**  
    Transaction validation frequently encounters malformed or invalid inputs submitted from untrusted external sources (network peers, user inputs). Panicking crashes the process or thread, causing potential service outages. Returning `Result<(), TransactionError>` provides structured, recoverable error handling, enabling callers to gracefully report validation failures, reject invalid network packets, or prompt users for corrections.

11. **How do enums help model regular and coinbase inputs?**  
    Regular inputs (which spend an existing outpoint) and coinbase inputs (which record block reward minting) carry distinct data fields. Rust's `InputKind` enum encapsulates both variants cleanly (`Regular { previous_output, value, sequence }` vs `Coinbase { block_height, reward }`). Exhaustive pattern matching enforces at compile-time that every function operating on transaction inputs explicitly handles both regular and coinbase scenarios.

12. **How does the `BitcoinValue` trait reduce duplication?**  
    `BitcoinValue` establishes a shared interface for any type representing a monetary bitcoin value by requiring a `.value() -> u64` method and providing a default `.value_in_btc() -> f64` implementation. This allows common valuation and conversion logic to be reused across `TxOutput`, `InputKind`, and `Utxo` without duplicating calculation methods.

## Design notes

### UTXO Selection Trade-offs

The `select_utxos` implementation uses a simple FIFO (slice-order) greedy strategy that iterates through available UTXOs and accumulates them until the required target value is reached.

- **Pros:** Fast ($O(N)$ time complexity), deterministic, simple to reason about, and preserves slice ordering without requiring allocations or sorting overhead.
- **Cons:** May leave sub-optimal change outputs, accumulate more inputs than necessary (increasing transaction size/fees), or fail to eliminate change outputs when an exact match is available.

### Advanced Selection Strategies (Bonus Analysis)

Production Bitcoin wallets (such as Bitcoin Core) implement more sophisticated coin selection algorithms:

1. **Branch and Bound (BnB / Knapsack)**: Searches for an exact combination of UTXOs that equals `target + fee` without producing change. By avoiding a change output, BnB saves output serialization bytes, reduces mining fees, and prevents UTXO set bloat.
2. **Single Random Draw (SRD)**: Randomly selects UTXOs until the target is satisfied, providing privacy benefits by avoiding deterministic input sorting that external observers could use to analyze wallet balances.
3. **Largest-First Selection**: Sorts UTXOs in descending order by value to minimize the total number of inputs required, reducing total transaction size and fee cost.
4. **Waste Metric Optimization**: Evaluates candidates based on a "waste metric" (comparing current fee rates against long-term baseline fee rates) to decide whether to consolidate small inputs or spend large UTXOs.

## Example output

```text
Transaction v2 (locktime: 0)
  Inputs (2): total 120000 sats
    [0] Regular(1111111111111111111111111111111111111111111111111111111111111111:0, 70000 sats, seq: 0xffffffff)
    [1] Regular(2222222222222222222222222222222222222222222222222222222222222222:1, 50000 sats, seq: 0xffffffff)
  Outputs (2): total 118000 sats
    [0] 90000 sats -> bc1qreceiver (P2wpkh)
    [1] 28000 sats -> bc1qsender (P2wpkh)
  Fee: 2000 sats
```
