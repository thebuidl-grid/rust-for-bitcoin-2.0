# Week 2 Assignment - Written Answers

Here are the answers to the assignment questions, the explanation for the ownership compiler error from Part 7, and my design notes.

---

## Written Answers

### 1. What is a Bitcoin transaction input?
It's a reference to a previous, unspent transaction output (a UTXO) that you want to spend. In the real Bitcoin network, it also includes a signature or witness data proving you actually own and have the right to spend those coins.

### 2. What is a Bitcoin transaction output?
It defines who gets the coins and how much they receive. It locks a specific amount of satoshis to a recipient's address or script, which then requires a matching signature to spend in a future transaction.

### 3. What is a UTXO?
Unspent Transaction Output. Basically, it's just a chunk of bitcoin sitting in a previous transaction's output that hasn't been spent yet. Your total wallet balance is just the sum of all the UTXOs you hold.

### 4. What does an outpoint identify?
An outpoint points to a specific UTXO. It has two parts: the transaction ID (`txid`) where the UTXO was created, and the index (`vout`) of that output in the transaction's outputs list.

### 5. How is a transaction fee calculated?
You subtract the total output value from the total input value: `Fee = Inputs - Outputs`. There isn't an explicit "fee" field in a Bitcoin transaction; whatever is left over automatically goes to the miner who mines the block.

### 6. Why use integers rather than floating-point numbers for bitcoin amounts?
Floats can introduce precision and rounding errors (like `0.1 + 0.2 = 0.30000000000000004`) due to how binary represents decimals. For money, you need absolute accuracy, so Bitcoin uses integers to count everything in satoshis (1 BTC = 100,000,000 sats) to prevent any rounding bugs.

### 7. Why does `total_input_value()` borrow `self`?
It only needs to read the inputs to sum their values. Because it doesn't modify the transaction or need to take ownership of it, we use a read-only shared borrow (`&self`).

### 8. Why does `add_input()` take `&mut self`?
Since it adds a new input and changes the `inputs` vector, it modifies the transaction state. This requires exclusive, mutable access (`&mut self`).

### 9. What happens when an input is moved into a transaction?
Ownership of the input shifts to the `Transaction` struct. Because of Rust's move semantics, the caller cannot access or modify that input variable anymore.

### 10. Why is `Result` preferable to `panic!` for validation failures?
A `panic!` crashes the entire program immediately, which is bad for a user or a running node. Returning a `Result` allows the calling code to handle the error gracefully—like showing a clean error message or asking the user to fix the inputs.

### 11. How do enums help model regular and coinbase inputs?
Coinbase inputs (which create new coins in blocks) and regular inputs have completely different fields (e.g. coinbase has block height, regular has previous outpoints). An enum (`InputKind`) lets us store both types in the same vector and forces us to write match arms for both, ensuring we never miss a case.

### 12. How does the `BitcoinValue` trait reduce duplication?
It provides a single interface (`value` and `value_in_btc`) for anything that represents an amount of bitcoin. Instead of writing separate value calculation functions for outputs and both kinds of inputs, we write it once in the trait and implement it.

---

## Part 7 Ownership Compiler Error

```text
error[E0507]: cannot move out of index of `Vec<TxOutput>`
   --> src\transaction.rs:164:17
    |
164 |     let first = transaction.outputs[0];
    |                 ^^^^^^^^^^^^^^^^^^^^^^ move occurs because value has type `TxOutput`, which does not implement the `Copy` trait
```

### What caused this?
The error happens because we tried to assign `transaction.outputs[0]` to a local variable `first`. Since `TxOutput` doesn't implement the `Copy` trait, this assignment attempts to pull ownership of the output out of the `Vec`. However, since `transaction` was passed as a read-only reference (`&Transaction`), we aren't allowed to take ownership of its internals. 

Also, trying to return a reference to `first` (`Some(&first)`) fails because `first` is a local variable that gets dropped when the function returns. The solution is to borrow directly from the vector (`&transaction.outputs[0]`) so we just return a reference that points back to the original transaction data.
