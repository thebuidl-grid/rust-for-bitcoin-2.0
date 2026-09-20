**1.What is a Bitcoin transaction input?
An input is a reference to money you already own (from a previous transaction) that you're now spending. It's like handing over cash you received earlier — you're proving you have coins to spend by pointing to where they came from.

**2. What is a Bitcoin transaction output?**
An output is where the money goes — a destination and an amount. Every transaction creates one or more outputs, each saying "this many sats go to this address."

**3. What is a UTXO?**
UTXO = "Unspent Transaction Output." It's an output from a past transaction that hasn't been spent yet — meaning it's still available to use as an input in a future transaction. Your wallet balance is really just the sum of all UTXOs you control.

**4. What does an outpoint identify?**
An outpoint pinpoints one specific output from one specific past transaction — it's the combination of a transaction ID (`txid`) and an index (`vout`, which output in that transaction, since a transaction can have several).

**5. How is a transaction fee calculated?**
Fee = total input value − total output value. Whatever's left over after all outputs are paid is the fee, collected by whoever mines the block.

**6. Why integers instead of floats for bitcoin amounts?**
Floating-point numbers lose precision with repeated arithmetic (rounding errors), which is unacceptable for money — you can't have a transaction be off by a fraction of a cent due to math. Integers (satoshis, the smallest unit) are exact, so add/subtract always gives a precise result.

**7. Why does `total_input_value()` borrow `self`?**
It only needs to *read* the data (sum up values) — it doesn't need to change anything. Borrowing (`&self`) lets it look at the data without taking ownership, so the caller can keep using `transaction` afterward.

**8. Why does `add_input()` take `&mut self`?**
Because it *changes* the transaction — it pushes a new input into the `inputs` vector. Mutating data requires a mutable reference (`&mut self`), unlike just reading it.

**9. What happens when an input is moved into a transaction?**
Ownership of that `InputKind` value transfers from wherever it was created into the `Transaction`'s `inputs` vector. The original variable can no longer be used — this is exactly what your compiler error in Part 7 demonstrated, just with a `TxOutput` instead of an input.

**10. Why is `Result` preferable to `panic!` for validation failures?**
`panic!` crashes the whole program immediately — no recovery, no graceful handling. `Result` lets the caller decide what to do with a failure (show an error message, retry, log it, etc.) without the program dying. Invalid user data (like a malformed transaction) is expected/recoverable, not a bug — so it should be an `Err`, not a crash.

**11. How do enums help model regular and coinbase inputs?**
An enum lets one type represent two genuinely different "shapes" of data — a `Regular` input has a previous output, value, and sequence; a `Coinbase` input has a block height and reward instead. Rust forces you to `match` every variant, so you can't accidentally forget to handle one case (like forgetting coinbase inputs need different treatment somewhere).

**12. How does the `BitcoinValue` trait reduce duplication?**
Instead of writing separate "get the value" logic wherever you need it, both `TxOutput` and `InputKind` implement one shared trait method (`value()`). Anything generic that needs "the value of a thing" can just call `.value()` regardless of which type it's given, and `value_in_btc()` is written once on the trait and works for both types automatically.

Now write your own one-to-two-sentence version of each — you understand this material, you built all the code yourself. Want me to also help you word the ownership-experiment explanation and the design-notes section next, or do you want to draft those yourself first?

## Submission standard

- All required TODOs are implemented; no required tests remain ignored.
- At least eight meaningful tests pass.
- `cargo fmt --check` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.
- `README.md` contains all written answers, ownership observations, design notes,
  and example output.
- Do not add an external Bitcoin library; the goal is to practise core Rust.
submission

error[E0382]: borrow of moved value: `out`
  --> src/main.rs:46:20
   |
40 |     let out = TxOutput {
   |         --- move occurs because `out` has type `TxOutput`, which does not implement the `Copy` trait
...
45 |     transaction.add_output(out);
   |                            --- value moved here
46 |     println!("{}", out.value);
   |                    ^^^^^^^^^ value borrowed here after move

   What caused it, in plain terms (you should put this in your own words in the README, but here's the concept to explain): add_output takes ownership of its output: TxOutput parameter (pub fn add_output(&mut self, output: TxOutput)), not a reference. So calling transaction.add_output(out) moves out into the function — out no longer belongs to the calling scope afterward. Since TxOutput contains a String (heap-allocated, not Copy), Rust won't silently duplicate it; ownership transfers instead. Trying to read out.value afterward is trying to use a variable that no longer owns its data, so the compiler rejects it at compile time rather than risking a use-after-move bug at runtime.
