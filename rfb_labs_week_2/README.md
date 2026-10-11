# Rust for Bitcoin 2.0 — Week 2

## Overview

This assignment implements a simplified Bitcoin transaction model in Rust to practise structs, enums, traits, ownership, borrowing, collections, and `Result`-based error handling.

This is an educational model. It does not serialize, sign, or broadcast real Bitcoin transactions. Monetary values are integer satoshis (1 BTC = 100,000,000 sats).

## Implementation

- Added transaction inputs and outputs, input/output totals, and fee calculation.
- Added descriptive transaction errors and validation for missing inputs/outputs, zero-value non-OP_RETURN outputs, insufficient input value, mixed input types, multiple coinbase inputs, and empty regular-input transaction IDs.
- Implemented `BitcoinValue` and `Display` for the required transaction types.
- Implemented borrowed highest-value output and recipient-filtering functions without cloning outputs.
- Implemented input-order UTXO selection using borrowed references and an insufficient-funds error.
- Added tests for regular and coinbase transactions, validation errors, output lookups, totals, fees, and UTXO selection.

## Written answers

1. **What is a Bitcoin transaction input?**
   An input references a previous output being spent and provides the information needed to authorize that spend. This assignment uses a simplified model.

2. **What is a Bitcoin transaction output?**
   An output specifies a value in satoshis and the recipient or locking-script type that determines how the value may be spent.

3. **What is a UTXO?**
   An unspent transaction output that can potentially be used as an input to a future transaction.

4. **What does an outpoint identify?**
   A specific output of a transaction, identified by its transaction ID and output index (`vout`).

5. **How is a transaction fee calculated?**
   The fee is total input value minus total output value, provided the outputs do not exceed the inputs.

6. **Why use integers rather than floating-point numbers for bitcoin amounts?**
   Integer satoshis represent amounts exactly and avoid floating-point rounding errors in monetary calculations.

7. **Why does `total_input_value()` borrow `self`?**
   It only reads input values, so borrowing allows it to calculate the total without consuming the transaction.

8. **Why does `add_input()` take `&mut self`?**
   It must mutate the transaction's input collection to append the new input.

9. **What happens when an input is moved into a transaction?**
   Ownership transfers to the transaction. The original variable cannot be used afterward unless the value is cloned or ownership is otherwise retained.

10. **Why is `Result` preferable to `panic!` for validation failures?**
    `Result` lets callers handle expected invalid data and errors explicitly rather than unexpectedly terminating execution.

11. **How do enums help model regular and coinbase inputs?**
    An enum represents distinct input variants. Pattern matching makes code handle the regular and coinbase cases explicitly.

12. **How does the `BitcoinValue` trait reduce duplication?**
    It provides a shared interface for retrieving satoshi values from different types, allowing totals to use common logic.

## Ownership experiment

The following temporary Rust example produced compiler error `E0382`:

```text
error[E0382]: borrow of moved value: `input`
  --> rfb_ownership_check.rs:4:16
   |
2  | let input = String::from("transaction input");
   |     ----- move occurs because `input` has type `String`, which does not implement the `Copy` trait
3  | let inputs = vec![input];
   |                   ----- value moved here
4  | println!("{input}");
   |            ^^^^^ value borrowed here after move
```

The `String` is moved into the vector, transferring ownership. Using `input` afterward is invalid because the original variable no longer owns the value. Borrowing it or deliberately cloning it would be alternatives, depending on the intended ownership.

## Design notes

- **Money:** Values are stored as integer satoshis.
- **Errors:** Validation and coin selection return `Result` errors instead of panicking for expected invalid input.
- **Borrowing:** Output lookup functions return references to existing outputs, avoiding unnecessary clones.
- **UTXO selection:** The basic algorithm selects UTXOs in input order until the target is met. It is simple and deterministic but does not minimize inputs or change.
- **Scope:** The optional Part 10 transaction-state model was not implemented because it is not required.

## Verification

Commands run from this directory:

```powershell
cargo test
cargo run
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

The tests, formatting check, and strict Clippy check passed in the recorded terminal run. After adding the additional required tests, `cargo test` passed with 12 integration tests and zero failures.

### Payment example output

```text
Transaction(version=2, locktime=0, inputs=2, outputs=2, total_input=120000 sats, total_output=118000 sats, fee=2000 sats)
```

The example spends 70,000 and 50,000 sats, pays 90,000 sats to `bc1qreceiver`, returns 28,000 sats to `bc1qsender`, and calculates a 2,000-sat fee.

## Scope and limitations

This is a simplified transaction model for learning Rust. It does not construct, sign, serialize, or broadcast a real Bitcoin transaction.
