# Rust for Bitcoin 2.0 — Week 2

A simplified Bitcoin transaction model, built to practise structs, enums,
traits, ownership, borrowing, collections and `Result`.

I did all ten parts including the optional Part 10.

```bash
cargo test      # 36 tests, none ignored
cargo run
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

## Layout

| File | What's in it |
| --- | --- |
| `src/transaction.rs` | Parts 1-3 and 5-7: the types, the methods, validation, traits, the borrowing functions |
| `src/error.rs` | Part 4: `Display` for each `TransactionError` |
| `src/utxo.rs` | Part 9: `select_utxos` |
| `src/main.rs` | Part 8: the payment |
| `src/state.rs` | Part 10: the state machine |
| `examples/ownership_experiment.rs` | Part 7: the move-then-use test |

## Written answers

### 1. What is a Bitcoin transaction input?

An input is how a transaction says which coins it wants to spend. It points at
an output from some earlier transaction using an `OutPoint`, and that output has
to still be unspent.

One thing I noticed: my `InputKind::Regular` stores the `value` on the input
itself, but a real Bitcoin input doesn't do that. A real node looks the amount up
from the UTXO set using the outpoint. It's on the struct here so we can total the
inputs without having a chain to query.

Coinbase inputs are the odd one out because they don't spend anything. They're
where new coins get created.

### 2. What is a Bitcoin transaction output?

The other half. An output creates new spendable coins: an amount in sats, plus
the rule for who's allowed to spend them. Real Bitcoin uses a script for that
rule, but here it's simplified down to a `recipient` string and an `OutputType`
saying which kind of script it would have been. Once the transaction confirms,
each output just sits there as a UTXO until someone uses it as an input.

### 3. What is a UTXO?

Unspent Transaction Output. An output that exists and hasn't been spent yet.

The part that surprised me is that Bitcoin doesn't store balances anywhere.
There's no account with a number in it. Your balance is just the sum of every
UTXO you hold the keys for, so a wallet is really just keeping track of a list of
these.

### 4. What does an outpoint identify?

One exact output. It's the txid of the transaction that created it plus which
output it was inside that transaction (`vout`, counting from 0). That's why my
`Display` prints it as `txid:vout`.

Because the pair is unique, two inputs pointing at the same outpoint means
someone is trying to spend the same coins twice. That's how a double spend gets
spotted.

### 5. How is a transaction fee calculated?

```text
fee = total inputs - total outputs
```

It isn't stored in the transaction anywhere, which I didn't expect. Whatever you
don't hand to an output is what the miner keeps.

That's why my `fee()` is just a subtraction. It's also why I had to use
`checked_sub`: if the outputs are bigger than the inputs, `u64` would wrap around
and hand back a massive fake fee instead of telling me the transaction is broken.

### 6. Why use integers rather than floating-point numbers for bitcoin amounts?

Because floats can't hold exact decimals. `0.1 + 0.2` gives you
`0.30000000000000004`, not `0.3`. That's bad enough for money on its own, and the
error builds up the more you add.

With Bitcoin it's worse, because every node has to get exactly the same answer.
If one node worked out a fee slightly differently to another, they'd disagree
about whether a transaction is even valid. So everything is counted in whole
satoshis and BTC is only used for showing to people. That's all `value_in_btc()`
is for, it's the last step before printing.

### 7. Why does `total_input_value()` borrow `self`?

Because it only reads, it doesn't change anything. `&self` is a shared borrow, so
the caller keeps their transaction and can call it as often as they like.

If it took `self` by value it would swallow the whole transaction just to give
back one number, which clearly isn't what you want. It also has to work this way
for `Display`, because that function already holds `&self`, so everything it
calls inside has to be fine with a shared borrow.

### 8. Why does `add_input()` take `&mut self`?

Because it pushes onto `self.inputs`, and you can't change something through a
shared reference.

`&mut` is exclusive, so while `add_input` is running nothing else can be looking
at the transaction. That isn't just a rule for the sake of it. Pushing to a `Vec`
can reallocate its memory, and if someone was holding a reference into that `Vec`
it would end up pointing at memory that's been freed. Rust catches it at compile
time instead.

### 9. What happens when an input is moved into a transaction?

`add_input` takes `InputKind` by value, and `InputKind` isn't `Copy` because it
has a `String` inside it. So it gets moved, not copied.

The transaction's `Vec` owns it now and my original variable is dead. The
compiler won't let me use it again (I tried on purpose, the error is below). It
also means the `String` only gets freed once, by the transaction, rather than
twice.

If I want to read it afterwards I have to go through the new owner, like
`transaction.inputs.first()`.

### 10. Why is `Result` preferable to `panic!` for validation failures?

Because a bad transaction isn't a bug in my code, it's just bad data. Nodes get
sent invalid transactions constantly and it would be terrible if that took the
process down.

`Result` puts the failure in the signature so you can't ignore it by accident,
and you can actually do something about it. A wallet can show "insufficient
funds" and let the user pick different coins instead of crashing. It carries
useful detail too, since `OutputsExceedInputs { total_inputs, total_outputs }`
tells you by how much. And it's much easier to test, I can just `assert_eq!` on
the error.

I'd keep `panic!` for things that are supposed to be impossible.

### 11. How do enums help model regular and coinbase inputs?

They hold genuinely different data. A regular input needs a previous outpoint. A
coinbase input doesn't have one at all, but it does have a block height and a
reward.

If I forced both into one struct I'd need `previous_output: Option<OutPoint>`,
and then everyone reading the code has to remember it's always `Some` except for
coinbase. That's exactly the sort of thing that goes wrong six months later. The
enum only lets you build the two shapes that actually make sense.

The other half is that `match` has to cover every variant. I couldn't write
`BitcoinValue for InputKind` without deciding what a coinbase input is worth,
because the compiler wouldn't let me finish. If someone adds a third variant
later they'll get errors pointing at every place that needs updating.

### 12. How does the `BitcoinValue` trait reduce duplication?

The match on the two input variants only exists in one place, inside
`impl BitcoinValue for InputKind`. `total_input_value`, the `Display` impls and
my tests all just call `.value()`.

Without the trait I'd have to repeat that match in each of those places and keep
them all in sync. This way a new variant means editing one function.

`value_in_btc()` helps in a different way. It's a default method on the trait, so
outputs and both input variants get it for free without me writing it three
times, and the `/ 100_000_000` only appears once in the whole crate.

## Ownership experiment (Part 7)

`examples/ownership_experiment.rs` moves an input into a transaction and then
tries to use it again. Uncomment the marked line and run
`cargo build --example ownership_experiment`:

```text
error[E0382]: borrow of moved value: `input`
  --> examples/ownership_experiment.rs:25:20
   |
11 |     let input = InputKind::Regular {
   |         ----- move occurs because `input` has type `InputKind`, which does not implement the `Copy` trait
...
22 |     transaction.add_input(input);
   |                           ----- value moved here
...
25 |     println!("{}", input.value());
   |                    ^^^^^ value borrowed here after move

For more information about this error, try `rustc --explain E0382`.
error: could not compile `rfb-labs-week-2` (example "ownership_experiment") due to 1 previous error
```

**What caused it.** `add_input` takes the input by value, not by reference.
`InputKind` can't be `Copy` because it contains a `String`, and a `String` owns
memory on the heap. If it got copied you'd have two things that both think they
own the same allocation, and it would get freed twice.

So Rust moves it instead. On line 22 the value goes into the transaction, and my
`input` variable stops being valid from that point on. Line 25 tries to use it
and the compiler stops me.

It took me a minute to stop reading this as the compiler being difficult. An
input really does belong to one transaction, so the rule matches how Bitcoin
actually works. The fix isn't to clone it either, it's to read it back through
whatever owns it now. That's why the Part 7 functions return `&TxOutput` instead
of `TxOutput`.

## Design notes

### Validation order

I check the rules in the same order ASSIGNMENT.md lists them so the two are easy
to compare side by side.

The over-spend check calls `self.fee()?` rather than writing the comparison out
again, so the two can't drift apart if I change one later. For the empty txid I
used `trim().is_empty()`, so a txid that's only spaces gets rejected too.

### Arithmetic

Totals use `saturating_add` and `fee()` uses `checked_sub`.

Neither can realistically overflow. Every bitcoin that will ever exist is about
2.1 quadrillion sats and `u64` goes up to roughly 18 quintillion. But a
transaction can be built from values I don't control, and I didn't want it to be
able to panic in debug or quietly wrap in release.

`fee()` returns `Result` rather than `Option` so the error can carry both totals.
That's what lets `Display` print `fee: unavailable (outputs spend 60000 sats but
the inputs only provide 50000 sats)` instead of just saying something went wrong.

### UTXO selection

I implemented what the assignment asks for. Walk the slice in order, keep adding
until the target is covered, stop as soon as it is. It returns `Vec<&Utxo>` so
the wallet keeps its own UTXOs and nothing gets copied.

It's simple and predictable, but it isn't what a real wallet should do. Three
problems I can see:

1. **It ignores fees.** Every extra input makes the transaction bigger, so it
   costs more to send. "Enough" should really mean enough for the payment plus
   the fee that those inputs cause, which is circular, because you don't know the
   fee until you know how many inputs you picked. I dodged this in `main.rs` by
   just deciding on 2,000 sats up front.
2. **It always creates change.** Whatever is left over becomes a change output.
   That costs bytes, it might be dust that costs more to spend than it's worth,
   and it keeps making the UTXO set bigger.
3. **It's predictable.** Always spending in the same order makes a wallet easy to
   recognise on chain, which is bad for privacy.

From reading about how Bitcoin Core handles this, the better approach is Branch
and Bound. It looks for a combination that lands just above the target, close
enough that you don't need a change output at all. That saves the bytes, and more
importantly it gets rid of the change address, which is one of the main things
chain analysis uses to link someone's transactions together. If it can't find a
match it falls back to picking at random, then compares the candidates by how
much they waste.

Sorting by value descending would be a simpler improvement, since fewer and
bigger inputs means a smaller transaction. But it burns through your large UTXOs
first and leaves you with a wallet full of dust.

I kept the simple version because that's what the assignment specifies and the
supplied tests check for it.

### Part 10, transaction state

`src/state.rs` goes Created -> Validated -> Signed -> Broadcast -> Confirmed, and
anything that hasn't finished can drop out to Rejected. Confirmed and Rejected
are the end of the line.

The `state` field is private, so the only way to change it is `advance_to`. If
you ask for a transition that isn't allowed it hands back
`Err(InvalidTransition)` and leaves the state exactly where it was, so a refused
move can't leave things in a weird half-changed state.

I looked at doing this with the typestate pattern instead, where each state is
its own type and the transitions consume `self`. That would catch mistakes at
compile time rather than runtime, which is stronger. I didn't go with it because
the states need to be stored in collections and compared, and that gets awkward
when every state is a different type. The enum stays `Copy` and comparable, and
`advance_to` is still the only way in.

### What I added

I didn't change any public name or signature from the starter code. I added
`Display for OutputType` (so `TxOutput`'s `Display` has something to print for
the script type), the `state` module, and `examples/ownership_experiment.rs`.

## Example output

`cargo run` spends the 70,000 and 50,000 sat UTXOs, pays 90,000 to
`bc1qreceiver`, works out the 28,000 change back to `bc1qsender`, and leaves
2,000 as the fee:

```text
Selected 2 UTXO(s):
  - 9f2c4a1b3e5d70819f2c4a1b3e5d70819f2c4a1b3e5d70819f2c4a1b3e5d7081:0 worth 70000 sats
  - 6d4e2f8a0c1b39576d4e2f8a0c1b39576d4e2f8a0c1b39576d4e2f8a0c1b3957:1 worth 50000 sats

Transaction v2 (locktime 0)
  2 input(s) totalling 120000 sats
    - regular 9f2c4a1b3e5d70819f2c4a1b3e5d70819f2c4a1b3e5d70819f2c4a1b3e5d7081:0 worth 70000 sats (0.00070000 BTC), sequence 4294967295
    - regular 6d4e2f8a0c1b39576d4e2f8a0c1b39576d4e2f8a0c1b39576d4e2f8a0c1b3957:1 worth 50000 sats (0.00050000 BTC), sequence 4294967295
  2 output(s) totalling 118000 sats
    - 90000 sats (0.00090000 BTC) to bc1qreceiver [P2WPKH]
    - 28000 sats (0.00028000 BTC) to bc1qsender [P2WPKH]
  fee: 2000 sats

Largest output: 90000 sats (0.00090000 BTC) to bc1qreceiver [P2WPKH]
```

## Tests

36 tests over three files, none left ignored.

`tests/transaction.rs` (22) covers a valid regular transaction and a valid
coinbase one, the totals, the fee including the zero-fee case and the underflow
case, all seven validation errors, the OP_RETURN exception for zero-value
outputs, `BitcoinValue` on both input variants and on outputs, the `Display`
summary and what it prints when the fee is invalid, and both Part 7 functions
including their empty cases.

`tests/utxo.rs` (7) covers a successful selection, insufficient funds, stopping
early once the target is met, an exact match, an empty wallet and a zero target.
The last one uses `std::ptr::eq` to check the returned UTXOs really are borrowed
from the caller's slice and not copies.

`tests/state.rs` (7) covers the happy path, trying to skip a stage, rejecting
from each non-terminal state, both terminal states, and the error message.
