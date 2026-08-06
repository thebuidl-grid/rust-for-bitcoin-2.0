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
```bash
xoulomon@xoulomon-ThinkPad-L13-Gen-2a:~/Desktop/rust4BTC/rust-for-bitcoin-2.0/rfb_labs_week_2$ cargo test
   Compiling rfb-labs-week-2 v0.1.0 (/home/xoulomon/Desktop/rust4BTC/rust-for-bitcoin-2.0/rfb_labs_week_2)
error: expected type, found lifetime
   --> src/transaction.rs:190:18
    |
190 |     transaction: 'a Transaction,
    |                  ^^ expected type
    |
help: you might have meant to write a reference type here
    |
190 |     transaction: &'a Transaction,
    |                  +

error[E0106]: missing lifetime specifier
   --> src/transaction.rs:182:65
    |
182 | pub fn highest_value_output(transaction: Transaction) -> Option<&TxOutput> {
    |                                                                 ^ expected named lifetime parameter
    |
    = help: this function's return type contains a borrowed value, but there is no value for it to be borrowed from
help: consider using the `'static` lifetime, but this is uncommon unless you're returning a borrowed value from a `const` or a `static`
    |
182 | pub fn highest_value_output(transaction: Transaction) -> Option<&'static TxOutput> {
    |                                                                  +++++++
help: instead, you are more likely to want to change the argument to be borrowed...
    |
182 | pub fn highest_value_output(transaction: &Transaction) -> Option<&TxOutput> {
    |                                          +
help: ...or alternatively, you might want to return an owned value
    |
182 - pub fn highest_value_output(transaction: Transaction) -> Option<&TxOutput> {
182 + pub fn highest_value_output(transaction: Transaction) -> Option<TxOutput> {
    |

error[E0308]: mismatched types
   --> src/transaction.rs:192:6
    |
189 | pub fn find_outputs_for_recipient<'a>(
    |        -------------------------- implicitly returns `()` as its body has no tail or `return` expression
...
192 | ) -> Vec<&'a TxOutput> {
    |      ^^^^^^^^^^^^^^^^^ expected `Vec<&TxOutput>`, found `()`
    |
    = note: expected struct `Vec<&'a TxOutput>`
            found unit type `()`
```
Explanation
1. Line 190 — 'a Transaction should be &'a Transaction. lifetime alone where a reference type was needed.
2. Line 182 — highest_value_output takes transaction by value but returns Option<&TxOutput>. A borrow needs something to borrow from — but the function owns transaction and it gets dropped at the end, so the reference would dangle.



1. What is a Bitcoin transaction input?

Ans- Transaction input of type InputKind is an enum spending either a previous output (Regular, referencing an OutPoint) or newly minted coins (Coinbase).

2. What is a Bitcoin transaction output?

Ans- Transaction output is of type TxOutput a value, recipient, and output_type that becomes spendable by someone.

3. What is a UTXO?

Ans- UTXO is an unspent TxOutput's value paired with the OutPoint that created it, sitting in the available_utxos pool until some future input spends it.

4. What does an outpoint identify?

Ans- Outpoint identifies one specific prior output by txid + vout index

5. How is a transaction fee calculated?


Ans- Transaction::fee() computes total_input_value() - total_output_value(), erroring with OutputsExceedInputs if outputs exceed inputs.

6. Why use integers rather than floating-point numbers for bitcoin amounts?

Ans- Integers over floats — value: u64 sats avoid floating-point rounding drift, which is unacceptable when every sat must balance exactly.

7. Why does `total_input_value()` borrow `self`?

Ans- total_input_value() borrows self because it only reads self.inputs to get values and their sum, so &self is enough and avoids taking ownership of the whole Transaction.

8. Why does `add_input()` take `&mut self`?

Ans- add_input() takes &mut self because it mutates self.inputs by pushing onto the Vec.

9. What happens when an input is moved into a transaction?

Ans- Mfunction add_input(&mut self, input: InputKind) takes input by value, so ownership transfers from the caller into self.inputs, and the caller can no longer use the original input.

10. Why is `Result` preferable to `panic!` for validation failures?

Ans- Validation failures like NoInputs or ZeroValueOutput are expected, recoverable conditions (TransactionError variants), so callers should get a Result to handle rather than have the process crash.

11. How do enums help model regular and coinbase inputs?

Ans- Enums for regular/coinbase inputs — InputKind lets one type represent two variants (Regular { previous_output, value, sequence } vs Coinbase { block_height, reward }), forcing every match to handle both instead of relying on optional/nullable fields.

12. How does the `BitcoinValue` trait reduce duplication?

Ans- BitcoinValue trait — value()/value_in_btc() are implemented once for the sats→BTC conversion logic exists in a single default method instead of being duplicated in each type.

## Design notes

Describe any choices you made, including your UTXO-selection trade-offs and (if
attempted) the optional transaction-state extension.

UTXO-selection design notes:

- Strategy: first-fit in slice order — walk available_utxos front to back, greedily accumulating until sum >= target, rather than sorting or optimizing for fewest inputs/least change. Simple and deterministic, but not fee- or change-optimal.
- Borrow, don't clone — returns Vec<&Utxo>, referencing the caller's slice instead of copying Utxo values.
- Overshoot allowed — stops as soon as the target is met/exceeded, so the selected total can exceed target; leftover becomes change, left to the caller to handle.
- Failure as Result — insufficient funds returns Err(InsufficientFunds { available, required }) instead of panicking, letting callers recover.

Trade-off: simplicity and predictability over efficiency — no attempt to minimize input count or wasted value, unlike smarter selection algorithms (e.g. largest-first, branch-and-bound).

## Example output

Paste the output of `cargo run` here once Part 8 is complete.
```bash
xoulomon@xoulomon-ThinkPad-L13-Gen-2a:~/Desktop/rust4BTC/rust-for-bitcoin-2.0/rfb_labs_week_2$ cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
     Running `target/debug/rfb-labs-week-2`
Transaction { version: 2, locktime: 0, inputs: 2, outputs: 2, total_input: 120000 sats, total_output: 118000 sats, fee: 2000 sats }
```

