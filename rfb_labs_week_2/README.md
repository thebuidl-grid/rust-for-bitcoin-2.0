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
