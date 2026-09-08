# Part 7 — Questions

## The ownership compiler error

This is the actual `cargo build` error produced by this code (moving a
`TxInput` into `Transaction::add_input`, then trying to use it again
afterward):

```rust
let mut tx = Transaction::new("tx1", 0);

let input = TxInput::Regular { previous_output: OutPoint::new("prev", 0) };
tx.add_input(input);

println!("added {input:?}");
```

```text
error[E0382]: borrow of moved value: `input`
 --> examples/ownership_error.rs:9:22
  |
6 |     let input = TxInput::Regular { previous_output: OutPoint::new("prev", 0) };
  |         ----- move occurs because `input` has type `TxInput`, which does not implement the `Copy` trait
7 |     tx.add_input(input);
  |                  ----- value moved here
8 |
9 |     println!("added {input:?}");
  |                      ^^^^^ value borrowed here after move
  |
help: consider cloning the value if the performance cost is acceptable
  |
7 |     tx.add_input(input.clone());
  |                       ++++++++

For more information about this error, try `rustc --explain E0382`.
```

**What caused it:** `add_input(&mut self, input: TxInput)` takes `TxInput`
by value, not by reference. Passing `input` into it moves ownership of that
value into the function (and from there into `self.inputs`). `TxInput`
holds an `OutPoint`, which owns a `String` (`txid`) — heap data — so Rust
can't silently duplicate it the way it would for a plain number. Once
ownership moves, the compiler invalidates the original `input` binding;
using it afterward (even just to print it) is a compile error, not a
runtime bug. The fix is either to stop using `input` after the move, or to
pass `input.clone()` if the caller genuinely needs to keep its own copy.

## Answers

**What is a Bitcoin transaction input?**
A pointer to money the transaction is spending. In real Bitcoin it's a
reference to a previous transaction's output plus proof you're allowed to
spend it. In this project, `TxInput` is an enum: `Regular` (points at an
existing UTXO through an `OutPoint`) or `Coinbase` (mints new coins instead
of spending anything).

**What is a Bitcoin transaction output?**
A chunk of value created by a transaction and locked to an address, sitting
there until someone spends it later. Here that's `TxOutput { value, address }`.

**What is a UTXO?**
"Unspent Transaction Output" — an output that hasn't been spent yet. Only
unspent outputs can be used as inputs to a new transaction. `UtxoSet` is
just the collection of these that a wallet (or the network) currently
considers spendable.

**What does an outpoint identify?**
One specific output of one specific past transaction: the transaction's id
plus the index of that output within it (`OutPoint { txid, index }`). It's
the "address" of a UTXO — not to be confused with a wallet/payment address.

**How is a transaction fee calculated?**
`fee = total input value − total output value`. Whatever value comes in
from the spent UTXOs but doesn't come back out in new outputs is the fee,
and it goes to whoever mines the transaction. `Transaction::fee` computes
exactly that.

**Why use integers rather than floating-point numbers for bitcoin amounts?**
Floating-point numbers lose precision with repeated addition/subtraction,
and money can't afford rounding errors. Bitcoin amounts are stored as a
whole count of satoshis (the smallest unit, `u64`), so there's nothing to
round — and integer overflow/underflow can be caught explicitly with
`checked_add`/`checked_sub`, which isn't possible the same way with floats.

**Why does `total_input_value()` borrow `self`?**
It only needs to read the transaction's inputs, not take ownership of the
transaction. Borrowing with `&self` means you can call it (and still use
the transaction afterward — print it, validate it, advance its status)
instead of it being consumed after one use.

**Why does `add_input()` take `&mut self`?**
It needs to change the transaction (push onto `self.inputs`) but doesn't
need to own it outright. `&mut self` lets you build a transaction up one
input/output at a time in place, rather than having to move it out and
reconstruct a new one on every call.

**What happens when an input is moved into a transaction?**
Ownership of that `TxInput` value transfers into the transaction's
`inputs` vector. The variable you had it in is no longer usable — the
compiler enforces this at compile time, as shown above, instead of letting
you accidentally use a value that's already "given away."

**Why is `Result` preferable to `panic!` for validation failures?**
`panic!` crashes the whole program with no way to recover. A wallet trying
to spend more than it has, or build a transaction with no outputs, isn't a
bug in the program — it's an expected, normal outcome that the caller
should be able to handle (show an error, ask for a smaller amount, retry).
`Result` turns that into an ordinary value you can match on, instead of a
crash.

**How do enums help model regular and coinbase inputs?**
A regular input needs an `OutPoint`; a coinbase input only needs a block
height. An enum lets each variant hold only the data it actually needs, and
because `match` has to be exhaustive, the compiler forces every place that
handles a `TxInput` (validation, fee calculation, printing) to explicitly
account for both cases — it's not possible to forget the coinbase case and
accidentally treat it like it points at a UTXO.

**How does the `BitcoinValue` trait reduce duplication?**
`TxOutput` and `Utxo` both "have a satoshi value," just stored slightly
differently. Rather than writing a separate summing loop for each type
(once for transaction outputs, once for UTXOs), both implement
`value_sats()` from `BitcoinValue`, so one generic function (`total_value`)
can sum either kind of collection.
