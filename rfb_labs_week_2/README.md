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


### Ownership experiment (Part 7)

Attempting to use a value after moving it into `add_input` produces:

​```text
error[E0382]: borrow of moved value: `utxo1`
  --> src/main.rs:18:16
   |
 8 |     let utxo1 = InputKind::Regular {
   |         ----- move occurs because `utxo1` has type `InputKind`, which does not implement the `Copy` trait
...
17 |     transaction.add_input(utxo1);
   |                           ----- value moved here
18 |     println!("{utxo1:?}");
   |                ^^^^^ value borrowed here after move
​```

`add_input` takes its parameter as `input: InputKind` — by value, not by reference — so calling
`transaction.add_input(utxo1)` moves `utxo1` into the function, and from there into
`transaction.inputs`. After that line, the local binding `utxo1` no longer owns anything: the
value it used to refer to now belongs to the `Transaction`. The next line tries to read `utxo1`
again via `println!`, but Rust's ownership rules only allow a value to have one owner at a time,
so the compiler rejects the second use at compile time rather than letting it silently reference
already-relocated memory.

The root cause is that `InputKind` doesn't implement `Copy`: one of its variants (`Regular`)
contains an `OutPoint`, which contains a `String` (`txid`). `String` owns a heap allocation, which
can't be implicitly duplicated by a cheap bit-copy — so Rust must treat assignment/passing of an
`InputKind` as a move, never an automatic copy. This is precisely the mechanism `add_input`'s
signature (Part 3) relies on to "transfer ownership": the compiler enforces it, not just a comment
or convention.


## Written answers

Answer in your own words. Add the ownership compiler error from Part 7 as a fenced
text block, then explain what caused it.

1. What is a Bitcoin transaction input?
   A reference to value being consumed by this transaction. `InputKind` models it as one of two mutually exclusive shapes: a `Regular` input, which points at a specific previous output (via `OutPoint`) and carries that output's value and a sequence number or a `Coinbase` input, the one special input per block that creates new coins from nothing, carrying a block height and reward instead of a previous output.

2. What is a Bitcoin transaction output?
   A destination for value being created by this transaction. Modeled by `TxOutput`: an amount (`value`, in satoshis), a `recipient`, and an `output_type` describing what kind of script locks it. An output only becomes spendable — turns into a UTXO — once the transaction containing it is accepted onto the chain.

3. What is a UTXO?
   An Unspent Transaction Output — an output from some previous transaction that hasn't yet been consumed as an input anywhere. It's the fundamental "coin" unit in Bitcoin's accounting model. Modeled here by `Utxo` (`outpoint` + `value`);
   `select_utxos` treats a slice of them as the pool of spendable money available to build a new transaction from.

4. What does an outpoint identify?
   One specific, unique previous output: which transaction it came from (`txid`) and which output index within that transaction(`vout`). 
   `txid:vout` is a coordinate system — a single `txid` can have many outputs, so `vout` disambiguates exactly one of them.

5. How is a transaction fee calculated?
   `fee = sum(inputs) - sum(outputs)`. 
   It is never a value written directly into the transaction. `fee()` computes it as `total_input_value() - total_output_value()` using `checked_sub`, so an impossible negative result (outputs worth more than inputs) produces a typed `OutputsExceedInputs`error instead of an integer-underflow panic.

6. Why use integers rather than floating-point numbers for bitcoin amounts?
    Floating-point numbers can't represent every decimal value exactly in binary, so repeated arithmetic on `f64` amounts accumulates rounding error — unacceptable when the numbers represent real money and totals must balance exactly. Satoshis are the smallest indivisible unit (1 BTC = 100,000,000 sats), so representing amounts as `u64` satoshi counts means every value is an exact integer: addition and subtraction never drift, and equality checks are always reliable.

7. Why does `total_input_value()` borrow `self`?
   It only needs to read the transaction's existing data (iterate `inputs`, sum their values) — it never changes anything. 
   An immutable borrow (`&self`) is the minimum access the function actually needs, and it lets the caller keep using the `Transaction` afterward instead of losing access to it.

8. Why does `add_input()` take `&mut self`?
   It is because it genuinely mutates the transaction , it pushes a new element onto `self.inputs`, growing the vector, which
   requires exclusive mutable access (`Vec::push` needs `&mut Vec`). An immutable `&self`wouldn't compile, and taking `self` by value would consume the whole transaction just to add one input.

9. What happens when an input is moved into a transaction?
   `add_input(&mut self, input: InputKind)` takes `input` by value, and
   `self.inputs.push(input)` moves it out of the local parameter and into the `Vec`. After that call, the caller's original binding no longer owns anything — the `InputKind` now lives solely inside `transaction.inputs`, and attempting to use the original variable
   again is a compile-time "use of moved value" error (exactly what the Part 7 ownership experiment demonstrated directly).

10. Why is `Result` preferable to `panic!` for validation failures?
    A validation failure (empty inputs, a malformed txid, outputs exceeding inputs) is an *expected*,recoverable condition, not a bug in the program. `panic!` immediately unwinds the whole program with no way for the caller to respond; `Result` lets the caller inspect exactly what went wrong via a specific `TransactionError` variant, decide how to handle it, and keep running. Because `TransactionError` also implements `Display`, an `Err` can be shown to a user as a clear message instead of crashing with a stack trace.

11. How do enums help model regular and coinbase inputs?
    `InputKind` expresses "exactly one of these two mutually exclusive shapes, and nothing else is representable" at the type level. A regular input and a coinbase input share no fields and are never a hybrid of both, so the enum makes invalid combinations structurally impossible rather than merely discouraged by convention. Combined with Rust's exhaustive `match`, every function that needs an input's value, or needs to validate or display it, is forced by the compiler to explicitly handle both variants — forgetting the coinbase case is a compile error, not a silent bug.

12. How does the `BitcoinValue` trait reduce duplication?
    Without it, every function needing "how much is this worth" would have to repeat the `match` distinguishing `InputKind::Regular`'s `value` field from `InputKind::Coinbase`'s `reward` field. By implementing `BitcoinValue::value()` once per type, that distinction lives in exactly one place; `total_input_value`, `fee`, and anything else needing a value just calls `.value()` (or passes the method itself as a function, e.g. `.map(InputKind::value)`) without knowing which variant it's looking at. It also gives `TxOutput` and `InputKind` a shared interface for free — the trait's default method `value_in_btc()` works identically on either type without either one writing its own conversion logic.

## Design notes

Describe any choices you made, including your UTXO-selection trade-offs and (if
attempted) the optional transaction-state extension.

## Example output

Paste the output of `cargo run` here once Part 8 is complete.
