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

### Ownership compiler error (Part 7)

```rust
fn consume(input: InputKind) {}

fn main() {
    let input = InputKind::Regular {
        previous_output: OutPoint { txid: "abcd".into(), vout: 0 },
        value: 50_000,
        sequence: u32::MAX,
    };
    consume(input);
    println!("{:?}", input); // error[E0382]: borrow of moved value
}
```

The error occurs because `consume` takes ownership of `input`. After the call,
`input` is no longer valid in the current scope. This is Rust's ownership system
ensuring there is exactly one owner for each value, preventing data races and
use-after-free bugs at compile time.

1. **What is a Bitcoin transaction input?**
   A Bitcoin transaction input references a previous transaction output (UTXO) that
   is being spent. It contains the outpoint (txid and vout) of the UTXO, the
   unlocking script (or witness), and a sequence number.

2. **What is a Bitcoin transaction output?**
   A Bitcoin transaction output creates new UTXOs by assigning a specific amount of
   satoshis to a recipient address with a locking script.

3. **What is a UTXO?**
   An Unspent Transaction Output (UTXO) is an output of a previous transaction that
   has not yet been spent. The entire Bitcoin UTXO set represents all spendable
   outputs in the network.

4. **What does an outpoint identify?**
   An outpoint uniquely identifies a specific output within a specific transaction,
   combining the transaction ID (txid) and the output index (vout).

5. **How is a transaction fee calculated?**
   The fee is the difference between the total value of all inputs and the total
   value of all outputs: `fee = sum(inputs) - sum(outputs)`. Miners claim this fee
   as reward.

6. **Why use integers rather than floating-point numbers for bitcoin amounts?**
   Floating-point numbers suffer from rounding errors and cannot precisely represent
   all decimal values. Using integer satoshis ensures exact arithmetic, deterministic
   validation, and avoids issues with precision loss.

7. **Why does `total_input_value()` borrow `self`?**
   Borrowing (`&self`) allows the method to read the transaction's data without
   taking ownership. This lets the caller continue using the transaction after the
   call, which is essential for querying state without consuming it.

8. **Why does `add_input()` take `&mut self`?**
   Adding an input modifies the transaction's internal `inputs` vector. `&mut self`
   provides exclusive mutable access, ensuring no other references exist during the
   mutation, which prevents data races and aliasing bugs.

9. **What happens when an input is moved into a transaction?**
   When `add_input` is called, ownership of the `InputKind` is transferred into the
   transaction's `inputs` vector. The caller can no longer access that input value
   because the transaction becomes its sole owner.

10. **Why is `Result` preferable to `panic!` for validation failures?**
    `Result` allows callers to handle expected failures gracefully. A `panic!`
    aborts the thread and is unrecoverable, which is inappropriate for expected
    business-logic errors like insufficient funds. `Result` forces explicit error
    handling.

11. **How do enums help model regular and coinbase inputs?**
    The `InputKind` enum makes the two distinct input types explicit at the type
    level. Pattern matching forces every code path to handle both variants, ensuring
    validation logic cannot accidentally forget one case.

12. **How does the `BitcoinValue` trait reduce duplication?**
    The `BitcoinValue` trait abstracts over any type that has a satoshi value.
    Both `TxOutput` and `InputKind` implement it, so generic code like fee
    calculation and value summation can operate on either without duplicating
    logic.

## Design notes

### UTXO selection trade-offs

The `select_utxos` function uses a simple greedy algorithm that selects UTXOs in
slice order until the target amount is reached. This is deterministic and O(n),
but it may over-select funds compared to more sophisticated algorithms like
largest-first or branch-and-bound. For this assignment, simplicity and correctness
were prioritized over optimization.

### Transaction state extension (optional)

I did not implement the optional Part 10 state machine because the core assignment
Parts 3–9 already exercise the required Rust concepts. If extended, states would
be modeled as an enum and transitions validated in a method that checks the
current state before allowing a change.

## Example output

```
Transaction v2 (locktime 0)
  Inputs: 2 (total: 120000 sats)
  Outputs: 2 (total: 118000 sats)
  Fee: 2000 sats
```
