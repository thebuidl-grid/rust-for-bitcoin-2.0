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

1. **What is a Bitcoin transaction input?** Basically a pointer to money
   that already exists — it says "I'm spending this specific output from
   an earlier transaction." In the code that's `InputKind`, and it's one
   of two things: a `Regular` input, which points at a previous `OutPoint`
   and says how much value it's spending plus a sequence number, or a
   `Coinbase` input, which isn't spending anything at all — it's the
   miner's block reward, so instead it just carries a block height and a
   reward amount.

2. **What is a Bitcoin transaction output?** The other side of that — it's
   where new value gets created. `TxOutput` just says how many sats it
   holds, who it's going to (`recipient`), and what kind of output it is
   (`P2pkh`, `P2wpkh`, `P2tr`, or `OpReturn` for embedding data with no
   spendable value attached).

3. **What is a UTXO?** An output nobody has spent yet. Every transaction
   consumes some existing outputs as inputs and creates new ones; whatever
   hasn't been consumed makes up the UTXO set. `Utxo` here is just an
   `OutPoint` plus its value — the info you'd need to spend it later as an
   input.

4. **What does an outpoint identify?** One specific output: which
   transaction created it (`txid`) and which position it was in that
   transaction's list of outputs (`vout`). You need both numbers because a
   single transaction can create several outputs.

5. **How is a transaction fee calculated?** It's just what's left over:
   total value coming in from the inputs minus total value going out to
   the outputs. `fee()` does that subtraction with `checked_sub`, so if
   outputs somehow end up bigger than inputs, it hands back an
   `OutputsExceedInputs` error instead of wrapping around to a huge
   number.

6. **Why use integers rather than floating-point numbers for bitcoin
   amounts?** Floats can't represent most decimal fractions exactly, so if
   you kept adding and subtracting fractional BTC values, tiny rounding
   errors would creep in — not something you want when it's money.
   Working in whole satoshis as `u64` avoids that entirely: every value is
   a whole number, so the arithmetic is exact, and overflow/underflow can
   be checked for explicitly instead of quietly losing precision.

7. **Why does `total_input_value()` borrow `self`?** It only needs to look
   at the inputs and add up their values — it's not changing anything and
   doesn't need to own the transaction to do that. Taking `&self` means I
   can call it and still go on to call `total_output_value()` or `fee()`
   on the same transaction afterward, instead of the transaction getting
   consumed after a single call.

8. **Why does `add_input()` take `&mut self`?** Adding an input means
   pushing onto the `inputs` vector, which is a mutation, so it needs
   `&mut self` to be allowed to change the transaction in place. It
   doesn't need full ownership though — the caller wants to keep the
   transaction around and keep calling `add_input`/`add_output` on it
   again.

9. **What happens when an input is moved into a transaction?** Once you
   pass an `InputKind` into `add_input`, it's moved — the transaction's
   `inputs` vector now owns it. Whatever variable you had at the call site
   can't be used again after that; the compiler blocks it outright. I
   actually triggered this for an output in the experiment below.

10. **Why is `Result` preferable to `panic!` for validation failures?** Bad
    transaction data isn't a bug, it's just something that can normally
    happen — someone might build a transaction with no inputs, or outputs
    bigger than the inputs. `Result` lets `validate()` hand that back as a
    value the caller has to deal with (often just with `?`), instead of
    `panic!`, which would crash the whole program over input that's
    completely expected to show up.

11. **How do enums help model regular and coinbase inputs?** Making
    `InputKind` an enum means an input has to be one or the other, never
    some fuzzy in-between. Anywhere the code needs to read an input's
    value or print it, it has to `match` on both variants, and the
    compiler won't let a variant get forgotten — so there's no way to
    accidentally reach for a `Regular`-only field on a `Coinbase` input.

12. **How does the `BitcoinValue` trait reduce duplication?** Both
    `TxOutput` and `InputKind` need a `value()` in sats, and both want the
    same sats-to-BTC conversion. `BitcoinValue` lets each type implement
    `value()` its own way (an output just returns `self.value`, an input
    has to match on which variant it is), but `value_in_btc()` only has to
    be written once, as a default method, and both types get it for free.
    It also means generic code — like `total_input_value()`'s
    `.map(BitcoinValue::value).sum()` — can call `.value()` on anything
    that implements the trait without caring what the concrete type is.

### Ownership experiment (Part 7)

To actually see this happen instead of just reasoning about it, I built a
`TxOutput`, moved it into a transaction with `add_output`, and then tried to
use it again afterward:

```rust
let mut transaction = Transaction::new(2, 0);

let change_output = TxOutput {
    value: 28_000,
    recipient: "bc1qsender".into(),
    output_type: OutputType::P2wpkh,
};

// `add_output` takes `TxOutput` by value, so this moves `change_output`
// into the transaction's `outputs` vector.
transaction.add_output(change_output);

// Using `change_output` here after it has been moved does not compile.
println!("{}", change_output.value);
```

And that refused to compile:

```text
error[E0382]: borrow of moved value: `change_output`
  --> examples/ownership_experiment.rs:17:20
   |
 6 |     let change_output = TxOutput {
   |         ------------- move occurs because `change_output` has type `TxOutput`, which does not implement the `Copy` trait
...
14 |     transaction.add_output(change_output);
   |                            ------------- value moved here
...
17 |     println!("{}", change_output.value);
   |                    ^^^^^^^^^^^^^^^^^^^ value borrowed here after move

For more information about this error, try `rustc --explain E0382`.
```

The reason this happens: `TxOutput` holds a `String` (`recipient`), which
lives on the heap, so `TxOutput` doesn't derive `Copy`. That means passing
it to `add_output(&mut self, output: TxOutput)` by value moves it —
ownership transfers over to the `Vec<TxOutput>` inside `transaction`. After
that, the original `change_output` binding is dead; the borrow checker
catches the later `println!` at compile time instead of letting a
use-after-move bug slip through at runtime. This is basically the whole
reason `add_input`/`add_output` are written to take ownership in the first
place (Part 3): once something's added, the transaction is the only owner
of it, so there's no way the caller can still be holding onto — or
accidentally mutating — data the transaction now depends on.

## Design notes

- **`validate()` reuses `fee()` instead of redoing the math.** Rather than
  comparing input/output totals a second time by hand, the last check in
  `validate` just calls `self.fee()?`. `fee()` already does the checked
  subtraction that separates a valid fee from `OutputsExceedInputs`, so
  `validate` gets that check — and the `?`-based error propagation — for
  free instead of duplicating it.
- **`Display` for `Transaction` won't panic on a bad fee.** A transaction
  can exist in an invalid state (outputs bigger than inputs) before
  anyone's called `validate()` on it, so `fmt::Display` matches on
  `self.fee()` and prints something like `invalid (<reason>)` instead of
  unwrapping — printing a transaction is always safe, valid or not.
- **UTXO selection trade-offs (Part 9).** `select_utxos` goes with the
  simplest strategy there is: walk the slice in the order it's given,
  keep adding UTXOs until the running total hits the target, and return
  `InsufficientFunds` (with the real total available) if the whole slice
  still isn't enough. It's easy to follow and it borrows instead of
  cloning, but it's not smart about money at all — it ignores value
  entirely. Given `[1, 1, 1, 100_000]` and a target of `100_000`, it would
  happily add up all four (1 + 1 + 1 + 100_000) instead of noticing the
  single large UTXO would've covered it alone. It also doesn't try to
  minimize the change output or the number of inputs picked, both of
  which matter for the eventual fee once real transaction weight comes
  into play. A more realistic algorithm would sort by value first —
  largest-first to keep the input count down, or something closer to
  Bitcoin Core's branch-and-bound coin selection — trading a bit more
  selection-time work for a cheaper resulting transaction.
- **Part 10 (transaction state) — didn't attempt it.** It was marked
  optional and I focused my time on getting Parts 1–9 solid instead.

## Example output

```text
$ cargo run
Transaction is valid.
Transaction v2 (locktime 0): 2 input(s), 2 output(s), total_in=120000 sats, total_out=118000 sats, fee=2000 sats
```
