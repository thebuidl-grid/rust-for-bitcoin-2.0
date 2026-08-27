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

1. What is a Bitcoin transaction input?
2. What is a Bitcoin transaction output?
3. What is a UTXO?
4. What does an outpoint identify?
5. How is a transaction fee calculated?
6. Why use integers rather than floating-point numbers for bitcoin amounts?
7. Why does `total_input_value()` borrow `self`?
8. Why does `add_input()` take `&mut self`?
9. What happens when an input is moved into a transaction?
10. Why is `Result` preferable to `panic!` for validation failures?
11. How do enums help model regular and coinbase inputs?
12. How does the `BitcoinValue` trait reduce duplication?

## Design notes

Describe any choices you made, including your UTXO-selection trade-offs and (if
attempted) the optional transaction-state extension.

## Example output

Paste the output of `cargo run` here once Part 8 is complete.


## Answers

- [ ] **Parts 1–2 — Data model:** Explain why `InputKind` is an enum and how matching forces both regular and coinbase inputs to be handled.
    = InputKind is an enum because the input can either be regular spend or a coinbase creation and an enum is like a switch in rust which can either be Off or On it can't be both On and Off or neither On or Off.An enum lets each variant carry only its own fields while in a case of a struct lets assume the InputKind is a struct, a struct with optional fields lets every instance carry every field.

    = Matching force you to handle both because match on an is checked at compile time,if you handle the Regular variant and forget to handle the Coinbase the compiler will refuse to build.That's the lang forcing you to think about the Coinbase variant every time you touch an input,rather than it being a bug later.

- [ ] **Part 7 — Borrowing:** implement `highest_value_output` and
  `find_outputs_for_recipient` using borrowed references without cloning. Complete
  the ownership experiment and record the compiler error in `README.md`.

        .0/rfb_labs_week_2/src$ cargo build
        Compiling rfb-labs-week-2 v0.1.0 (/home/celestine/Documents/RustforBitcoin/rust-for-bitcoin-2.0/rfb_labs_week_2)
        error[E0382]: borrow of moved value: `output`
        --> src/main.rs:14:20
        |
        7 |     let output = TxOutput {
        |         ------ move occurs because `output` has type `TxOutput`, which does not implement the `Copy` trait
        ...
        13 |     transaction.add_output(output);
        |                            ------ value moved here
        14 |      println!("{}",output.value);
        |                    ^^^^^^^^^^^^ value borrowed here after move

        For more information about this error, try `rustc --explain E0382`.
        error: could not compile `rfb-labs-week-2` (bin "rfb-labs-week-2") due to 1 previous error

    = So what happen here is add_output takes output: TxOutput by value, not by reference, so calling transaction.add_output(output) moves ownership of the TxOutput into the transaction's outputs vector. TxOutput contains a String (recipient), which doesn't implement Copy, so Rust can't silently duplicate it — after the move, the original output variable is no longer valid, and the compiler statically forbids reading from it. This is why println!("{}", output.value) fails at compile time rather than causing a runtime bug.

## Paste the output of `cargo run` here once Part 8 is complete.
**Part 8 — Payment:**

        Compiling rfb-labs-week-2 v0.1.0 (/home/celestine/Documents/RustforBitcoin/rust-for-bitcoin-2.0/rfb_labs_week_2)
            Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.17s
            Running `/home/celestine/Documents/RustforBitcoin/rust-for-bitcoin-2.0/rfb_labs_week_2/target/debug/rfb-labs-week-2`
        Transaction version 2
                    (locktime 0)

                    inputs: 2
                    (total 120000 sats)

                    outputs: 2
                    (total 118000 sats)

                    fee: 2000

- [ ] **Part 9 — Selection:** implement `select_utxos` over a borrowed slice. The
  basic algorithm selects in input order and returns borrowed UTXOs. Return
  `InsufficientFunds` when necessary. Bonus: justify a better selection algorithm.
    = The better algorithms will be largest-first (or Branch-and-Bound-lite) minimizes the number of inputs used, which directly reduces transaction size and therefore fees — since Bitcoin fees are priced per byte, not per bitcoin value.

## Design notes

Describe any choices you made, including your UTXO-selection trade-offs and (if
attempted) the optional transaction-state extension.

    = Validation order. validate() checks rules roughly cheapest-first: presence of inputs/outputs, zero-value non-OP_RETURN outputs, coinbase count/mixing, empty txids, then delegates the outputs-vs-inputs check to fee() via ? so that logic isn't duplicated.UTXO selection(Part9). select_utxos picks UTXOs in slice order until the target is met — simple, but can overshoot (e.g. selecting 120,000 sats for a 90,000 target) and doesn't optimize for fees or dust. A real wallet would use something like Bitcoin Core's Branch-and-Bound, or a simpler largest-first (fewer inputs, lower fees) or smallest-first (consolidates dust) strategy.Optional state extension (Part 10). src/state.rs adds a TxState enum and a StatefulTransaction wrapper. transition_to checks the (current, next) pair against a whitelist via matches!; anything not listed returns a new TransactionError::InvalidStateTransition instead of panicking or silently succeeding. Chose this runtime-checked approach over the typestate pattern for simplicity.

1. What is a Bitcoin transaction input? 
    A reference to a previous output being spent (a regular input), or a block reward being created (a coinbase input). It proves the sender has the right to spend that value.

2. What is a Bitcoin transaction output? 
    A destination for value — an amount and the conditions (address/script type) under which it can later be spent.

3. What is a UTXO? 
    An Unspent Transaction Output — an output that hasn't been used as an input yet. It's the "spendable balance" a wallet actually holds.

4. What does an outpoint identify?
    A specific previous output, via the txid of the transaction that created it and the vout index of which output within that transaction.

5. How is a transaction fee calculated?
    Total input value minus total output value. Miners keep the difference.

6. Why integers, not floats, for satoshis?
    Floats lose precision with repeated arithmetic, which is unacceptable for money — integer satoshis are exact and avoid rounding errors.

7. Why does total_input_value() borrow self? 
    It only needs to read the inputs to sum them; it doesn't need ownership, and borrowing lets the transaction keep being used afterward.

8. Why does add_input() take &mut self? 
    It mutates the transaction by pushing into self.inputs, so it needs a mutable reference to modify the struct in place.

9. What happens when an input is moved into a transaction? 
    Ownership transfers to the Vec inside the transaction; the original variable becomes invalid and the compiler refuses any further use of it.

10. Why Result over panic! for validation failures? 
    Invalid transactions are an expected, recoverable case, not a bug — Result lets the caller handle it gracefully, while panic! would crash the whole program over bad input.

11. How do enums help model regular and coinbase inputs? 
    InputKind forces every input to be exactly one variant or the other, and match requires both to be handled — making it impossible to represent or silently ignore an invalid combination.

12. How does BitcoinValue reduce duplication? 
    It gives both TxOutput and InputKind a shared .value() interface (plus a free .value_in_btc()), so other code can treat any "valued" item the same way instead of writing separate logic per type.
    