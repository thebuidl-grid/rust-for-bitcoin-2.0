# Week 2 Assignment Answers

## Parts 1–2: Data Model

`InputKind` is an enum because Bitcoin transactions have different input types
with different data requirements. A regular input references a previous output,
while a coinbase input contains block reward information.

Using `match` on `InputKind` forces every variant to be handled. This prevents
accidentally ignoring special cases such as coinbase transactions.

---

## Part 3: Transaction Methods

`add_input` and `add_output` take ownership of their arguments and move them
into the transaction vectors.

The fee calculation uses checked subtraction. If outputs exceed inputs, the
code returns `TransactionError::OutputsExceedInputs` instead of causing an
integer underflow.

---

## Part 5: Validation Rules

The validator checks:

- Transactions must contain inputs.
- Transactions must contain outputs.
- Zero-value outputs are rejected except OP_RETURN outputs.
- Outputs cannot exceed inputs.
- Coinbase and regular inputs cannot be mixed.
- Only one coinbase input is allowed.
- Regular inputs require non-empty TXIDs.

---

## Part 6: Traits

`BitcoinValue` provides a common way to retrieve satoshi values from outputs
and both input variants.

Display implementations provide readable representations for:

- OutPoint
- TxOutput
- InputKind
- Transaction

---

## Part 7: Borrowing

`highest_value_output` and `find_outputs_for_recipient` return references
borrowed from the original transaction.

No cloning is required because the returned data lives as long as the original
transaction.

Ownership experiment:

Attempting to move values out of a borrowed reference causes a compiler error
because Rust prevents moving ownership from borrowed data.

Example error:

cannot move out of borrowed content


---

## Part 8: Payment Example

Example transaction:

Version: 2
Locktime: 0

Inputs:
70,000 sats
50,000 sats

Outputs:
90,000 sats -> bc1qreceiver
28,000 sats -> bc1qsender

Fee:
2,000 sats


---

## Part 9: UTXO Selection

The implemented algorithm selects UTXOs in the order provided until the target
amount is reached.

A better production algorithm could choose UTXOs that minimize change,
transaction size, or fees.


