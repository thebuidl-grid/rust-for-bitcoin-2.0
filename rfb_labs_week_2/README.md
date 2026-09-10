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

### 1. What is a Bitcoin transaction input?
It contains the value that is going to be spent in a transaction. A transaction is going to use the necessary values to pay the outputs + fees. A input can only be spent using the a private key. An input contains the previous transaction id (txid), the vout (index of output used in the previous transaction), ScriptSig that unlocks the output to be spent.

### 2. What is a Bitcoin transaction output?
A output is the amount being sent and the ScriptPubKey, this script defines what must be met to spend this output later in the future. It is a locking mecanism. The ScriptPubKey can be,  for example: P2pkh, P2wpkh, P2tr.
When a transaction is made, the inputs are used and new outputs are created. In the next transaction the outputs are going to be inputs.

### 3. What is a UTXO?
UTXO stands for Unspent Transaction Outpout.
They are the value that can be spent in a transaction. When a transaction is created the inputs are used and new outputs are created. When any of these outputs were not spent in a transaction they are UTXO. 

### 4. What does an outpoint identify?
An outpoint identifies the transaction (txid) that created the output and the output’s index (vout) within that transaction . It specify the output among the transaction’s outputs.

### 5. How is a transaction fee calculated?
The transaction fee is calculated  by the sum of the inputs values minus the sum of the outputs values.

### 6. Why use integers rather than floating-point numbers for bitcoin amounts?
Because rounding problems can happen when working with floats.Different nodes and hardware can handle floats in diverse ways, which could create consensus problems. It’s safe to work with integers (sats) and avoid those issues.

### 7. Why does `total_input_value()` borrow `self`?
Because self will only be used to read the data, not modify or consume it. 

### 8. Why does `add_input()` take `&mut self`?
Because it is changing the inputs Vec, adding new values. This is why the mutable reference is being used. 

### 9. What happens when an input is moved into a transaction?
The ownership of the input is transfered, it does not exists anymore in the previous scope.

### 10. Why is `Result` preferable to `panic!` for validation failures?
`Result` gives callers control over how to handle errors, while `panic!` terminates the program. This makes Result the better choice for validation failures.

### 11. How do enums help model regular and coinbase inputs?
They help because an Input can only be Regular or Coinbase. They are mutually exclusive, even though both of them need to be treated as Inputs. Each one has its own fields. If a struct was used, we could end up with many Option fields unnecessarily.

### 12. How does the `BitcoinValue` trait reduce duplication?
It avoids duplication because it guarantees that any implementation of this trait will have the methods value() and value_in_btc(). value() must be implemented, while value_in_btc() has a default implementation. Once an object implements the trait, it can be treated as a BitcoinValue, which means that if a new object implements this trait, the old code doesn't need to be changed because generic functions already work with any BitcoinValue.


## Part 7 - Ownership Experiment - Compiler Error

```code
47 |     let input_example = InputKind::Regular {
   |         ------------- move occurs because `input_example` has type `InputKind`, which does not implement the `Copy` trait
...
55 |     tx.add_input(input_example);
   |                  ------------- value moved here
56 |
57 |     println!("{}", input_example);
   |                    ^^^^^^^^^^^^^ value borrowed here after move

For more information about this error, try `rustc --explain E0382`.
error: could not compile `rfb-labs-week-2` (bin "rfb-labs-week-2") due to 1 previous error
```

The problem occurs because tx.add_input gets the ownership of input_example, after that, when println tries to use input_example, the compiler gives a error. This happens because the input_example does not owns the object anymore, the ownership has been moved to tx.add_input.


## Design notes

Describe any choices you made, including your UTXO-selection trade-offs and (if
attempted) the optional transaction-state extension.

### Alternative coin selection algorithm: 

A better strategy would be to use "Largest First" or "Branch and Bound". Largest First  sorts inputs in descending order and selects the largest ones, with the advantage of resulting in fewer inputs (smaller transactions) but has the disadvantage of potentially leaving many small UTXOs unused. Branch and Bound finds the optimal combination that minimizes fee and change, with the advantage of being more efficient in terms of total fee but has the disadvantage of higher computational complexity.

### TransactionState 

enum TransactionState contains  the possible states: Created, Validated, Signed, Broadcast, Confirmed, Rejected. There is a implementation o fmt::Display that shows the corresponding state.
enum StateTransitionError is about invalid trasactions and wrong state. There is also a implementation of fmt::Display to this enum.
Transaction now has the field `pub state: TransactionState`. The method `current_state` shows the current state. `can_transition` confirm if a transition is possible. `transition_to` changes the current state.


## Example output

Part 8 `cargo run`  output.

```
cargo run

✓ Transaction is valid
Transaction (v2)
  Inputs (2):
    Regular input: 70000 sats from aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:0 (sequence: 4294967295)
    Regular input: 50000 sats from bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb:1 (sequence: 4294967295)
  Outputs (2):
    90000 sats to bc1qreceiver (P2wpkh)
    28000 sats to bc1qsender (P2wpkh)
  Total in: 120000 sats, Total out: 118000 sats
  Fee: 2000 sats
  Locktime: 0
``` 
