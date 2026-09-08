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
```                                                                                                    
┌──(julypjulius㉿kali)-[~/bitcoin-lightning-network/rust-for-bitcoin-2.0/rfb_labs_week_2]
└─$ cargo run
   Compiling rfb-labs-week-2 v0.1.0 (/home/julypjulius/bitcoin-lightning-network/rust-for-bitcoin-2.0/rfb_labs_week_2)
error[E0382]: borrow of moved value: `input`
  --> src/main.rs:17:20
   |
14 |     let input = InputKind::Coinbase { block_height: 1, reward: 5_000_000_000 };
   |         ----- move occurs because `input` has type `InputKind`, which does not implement the `Copy` trait
15 |
16 |     tx.add_input(input);
   |                  ----- value moved here
17 |     println!("{}", input.value()); // deliberately broken — triggers the error
   |                    ^^^^^ value borrowed here after move

For more information about this error, try `rustc --explain E0382`.
error: could not compile `rfb-labs-week-2` (bin "rfb-labs-week-2") due to 1 previous error

```
 The funtion add_input(&mut self, input: InputKind) takes its parameter by value, and not by reference. 
 Since InputKind derives neither Copy nor Clone,
 calling tx.add_input(input) doesn't hand the function a copy of input — it transfers ownership of the underlying data into Transaction's inputs: Vec<InputKind>. After that call, the original input binding in main is no longer valid; the compiler considers it "moved out. The next line, input.value(), tries to call a method that needs to read input's data — but that data has already been relocated into the Vec owned by tx. Rust's borrow checker catches this at compile time rather than allowing a use-after-move 
 We can fix this by calling input.value() before moving it into add_input.



1. What is a Bitcoin transaction input?

   It is a reference to a previous transaction output that's being spent 
   For instance in our implementation we have an enum 
   ``InputKind::Regular`` carries an ``Outpoit``(which txid/vout is spending),
    the value being spent , and a sequence of number. It's a proof of ownership of coins entering this transaction.

2. What is a Bitcoin transaction output?

   Refers to a destination value - an amount plus a locking condition(who can spend it later).
   In our code, Txoutput models this as value, recipient and output type(P2PKH, P2WPKH, P2TR or OP_RETURN for unspendable data).

3. What is a UTXO?

   It refers to an output that hasn't yet been consumed as some later transactions' input, 
   In our code we have a struct Utxo which is basically a TxOutput value paired with the Outpoint that identifies where it lives onchain,
    still available to spend

4. What does an outpoint identify?

   Outpoint identifes a specific, unique output: the txid of the transaction that created it,
   and the vout index of that output within that transactions' output list.

5. How is a transaction fee calculated?

   total_input_value - total_output_value. Miners keep the difference. 
   The fee() uses checked_sub specifically because if outputs ever exceed inputs, 
   plain subtraction on unsigned integers would underflow/panic — instead you return OutputsExceedInputs

6. Why use integers rather than floating-point numbers for bitcoin amounts?

   Floats can't represent most decimal fractions exactly — repeated arithmetic accumulates rounding errors, 
   which is unacceptable when real money is on the line.
   Satoshis as u64 integers are exact: every value is a whole number of the smallest unit, so addition/subtraction is exact and reproducible.

7. Why does `total_input_value()` borrow `self`?

   It only needs to read the inputs Vec to sum values — it doesn't need to own or mutate the transaction. 
   Taking ``&self`` lets callers use the transaction again afterward, 
   and lets us call it multiple times (e.g. from fee() and Display) without consuming anything.

8. Why does `add_input()` take `&mut self`?
   
   It takes a mutable reference reason being that it needs an exclusive access so that it can mutate the value 
   i.e the vector inputs by using the push method to add another input as ``self.inputs.push(input)``

9. What happens when an input is moved into a transaction?

   ``InputKind`` doesn't implement Copy, so passing it into ``add_input(input)`` by value transfers ownership — 
   the Vec<InputKind> inside Transaction now owns that value. 
   The caller's original binding is invalidated;
   that's why in main.rs I commented- it out to experiment the demonstration
   — trying to call ``input.value()`` after ``tx.add_input(input)`` fails to compile, because input no longer owns any data at that point.

10. Why is `Result` preferable to `panic!` for validation failures?

   Invalid transactions (empty inputs, mismatched fees, etc.) are expected, 
   and we need graceful error handling for these errors. panic! unwinds/aborts the program, 
   which is unacceptable for a library used by other code. 
   Result<T, TransactionError> forces callers to explicitly handle the failure case (via ?, match, etc.) and lets them decide what to do, without crashing.

11. How do enums help model regular and coinbase inputs?

   An enum expresses "exactly one of these variants, with different data per variant" — Regular has an OutPoint+value+sequence, Coinbase has a block_height+reward, and there's no way to accidentally construct an input with fields from both or neither. 
   Critically, match on an InputKind is exhaustive — the compiler forces one to handle both variants everywhere we inspect one (as in total_input_value, value(), Display), so it's structurally impossible to forget to handle coinbase inputs somewhere.
12. How does the `BitcoinValue` trait reduce duplication?
   The trait defines a shared behaviour and in the essence of our code its that the trait defines one value() method that TxOutput and InputKind each implement according to their own field names, plus a shared default value_in_btc() that every implementor gets for free by just implementing value(). Callers can write generic code against &dyn BitcoinValue or bound generics without caring which concrete type they have

## Design notes

Describe any choices you made, including your UTXO-selection trade-offs and (if
attempted) the optional transaction-state extension.

   select_utxos implements the simplest strategy: scan the slice in the order given, greedily accumulate UTXOs until the running total meets or exceeds the target, then stop. This doesn't try to minimize the number of inputs, avoid leaving dust-sized UTXOs behind, or minimize the resulting change amount.

Trade-offs of this approach:

Pro: simple, deterministic, O(n) — no sorting or backtracking needed.
Con: selection quality depends entirely on the caller's input order. If large UTXOs happen to come last, one might select many small ones and pay a higher fee (each input adds to transaction size/weight).
Con: it doesn't try to find an exact or near-exact match, so it tends to produce larger change outputs than necessary, which increases future UTXO fragmentation.

A better real-world algorithm  would be something like:

Largest-first: sort descending, pick fewest inputs → lower fee, but tends to accumulate small "dust" UTXOs over time since they're rarely picked.
Smallest-first / consolidation: sort ascending, prefer using up small UTXOs → keeps the UTXO set tidy but costs more in fees (more inputs).
Branch-and-bound (what Bitcoin Core  does): searches for a combination that produces zero or near-zero change, avoiding a change output entirely when possible — better privacy and lower long-term fees, at the cost of more complex selection logic.


### Part 10 — Transaction state extension (attempted)

I modeled the transaction lifecycle as a TransactionState enum (Created, Validated, Signed, Broadcast, Confirmed, Rejected) paired with a TrackedTransaction wrapper that holds a Transaction plus its current state. Rather than encoding valid transitions in the type system (a typestate pattern with a distinct type per state), I chose a runtime-checked approach: a single private transition() method holds the full list of allowed (from, to) state pairs, and every public transition method (mark_validated, mark_signed, mark_broadcast, mark_confirmed, mark_rejected)  through it. Any transition not explicitly listed as allowed returns a new TransactionError::InvalidStateTransition { from, to } instead of silently succeeding or panicking, consistent with the Result-based error handling used everywhere else in this crate.


One deliberate design choice: mark_validated() doesn't just flip the state label — it calls the existing Transaction::validate() first and only transitions to Validated if that succeeds. This means the state machine can't drift out of sync with the actual validity of the underlying transaction data. Confirmed and Rejected are both terminal states — neither appears as the "from" side of any allowed transition, so once a transaction reaches either, no further mark_* call can succeed.

This is covered by 8 tests in tests/state.rs, including the full happy-path walk through every state, two invalid-skip cases (Created → Signed and Created → Broadcast), both terminal states rejecting further transitions, Rejected being reachable from a non-Created state, and a case confirming mark_validated() fails when the underlying transaction itself is invalid.

## Example output

Paste the output of `cargo run` here once Part 8 is complete.

```
┌──(julypjulius㉿kali)-[~/bitcoin-lightning-network/rust-for-bitcoin-2.0/rfb_labs_week_2]
└─$ cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running `target/debug/rfb-labs-week-2`
Calculated fee: 2000 sats
Transaction is valid.
Transaction v2 (locktime 0)
  2 input(s), 2 output(s)
  total input: 120000 sats
  total output: 118000 sats
  fee: 2000 sats
```