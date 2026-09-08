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

```rust
error[E0382]: borrow of moved value: `input1`
  --> src/main.rs:17:20
   |
 8 |     let input1 = InputKind::Regular {
   |         ------ move occurs because `input1` has type `InputKind`, which does not implement the `Copy` trait
...
16 |     transaction.add_input(input1);
   |                           ------ value moved here
17 |     println!("{}", input1);
   |                    ^^^^^^ value borrowed here after move
```
*Explanation:* In Rust, variables have "ownership" over their data. When we pass `input1` into `add_input`, we "move" or give that data to the transaction. After giving it away, we don't own it anymore, so we can't print it. Trying to use it after giving it away causes this error!

1. What is a Bitcoin transaction input?
Think of an input like a digital gift card you received in the past. It's money that someone sent to you, and now you are "spending" it in a new transaction.

2. What is a Bitcoin transaction output?
An output is like a digital envelope holding money. You write someone's name (their address) on the envelope and put a specific amount of money inside. Now only they can open it.

3. What is a UTXO?
UTXO stands for "Unspent Transaction Output". It's simply an envelope of money that hasn't been opened or spent yet. Your Bitcoin "balance" is just the sum of all the unopened envelopes with your name on them.

4. What does an outpoint identify?
An outpoint is like a unique tracking code. It tells the Bitcoin network exactly which specific envelope of money (UTXO) you are trying to spend, by pointing to the exact past transaction that created it.

5. How is a transaction fee calculated?
It's just the leftover money. You take all the money you put into the transaction (Inputs) and subtract all the money you are sending to people (Outputs). Whatever is left over goes to the miner as a fee.

6. Why use integers rather than floating-point numbers for bitcoin amounts?
Computers are surprisingly bad at doing math with decimals (floating-point numbers) and often make tiny rounding mistakes. By using whole numbers (like "satoshis" instead of decimals of a Bitcoin), the math is always 100% perfect.

7. Why does `total_input_value()` borrow `self`?
Because it just wants to "look" at the transaction to count up the numbers. It doesn't need to change anything or take ownership of the transaction, so it just borrows it temporarily.

8. Why does `add_input()` take `&mut self`?
It needs to change (mutate) the transaction by adding a new input to it. The `&mut` means "I am borrowing this temporarily, but I am allowed to change it while I have it".

9. What happens when an input is moved into a transaction?
When you move an input into a transaction, you hand over complete ownership. It's like giving someone a physical object; once you hand it over, you can't use it anymore.

10. Why is `Result` preferable to `panic!` for validation failures?
If someone makes a mistake (like trying to spend more than they have), we just want to say "Oops, transaction failed!" and let the program keep running (`Result`). If we used `panic!`, the entire computer program would instantly crash and shut down.

11. How do enums help model regular and coinbase inputs?
An enum is like a box that can hold one of several specific things, but only one at a time. It lets us say "An input can be either a Regular input OR a Coinbase (miner reward) input". It keeps the code clean and forces us to write rules for both scenarios.

12. How does the `BitcoinValue` trait reduce duplication?
A trait is like a shared rulebook. Both Inputs and Outputs have a monetary value. Instead of writing separate code to get the value for each, the trait lets us say "Anything with this trait knows how to report its value". It saves us from writing the same code twice.

## Design notes

For picking which money to spend (UTXO selection), I used the simplest method: I just went down the list of available money and grabbed envelopes one by one until I had enough to pay the bill. It's like paying for groceries by taking bills out of your wallet one by one until you cover the total. It's super easy to understand and works perfectly!

For the fee calculation, I used `checked_sub`. This is a safe way to subtract numbers. If someone tries to spend more money than they put in, instead of the program crashing, it cleanly returns an `OutputsExceedInputs` error.

## Example output

```
Transaction (Version: 2, Locktime: 0)
Inputs (2): 120000 sats
Outputs (2): 118000 sats
Fee: 2000 sats
```
