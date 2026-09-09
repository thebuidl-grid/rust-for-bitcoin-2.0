# Lab 06 — Weight, virtual size, and fees

## Commands used

```bash
cargo test --test lab_06
bash grader/grade.sh
```

## Terminal output

running 4 tests

test calculates_bip141_weight ... ok

test rounds_weight_up_to_virtual_bytes ... ok

test reproduces_the_class_fee_comparison ... ok

test calculates_fee_from_feerate ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

## Evidence references


All four public tests in `tests/lab_06.rs` pass, covering:
- Computing BIP141 weight from stripped and total size, and rejecting an
  invalid size pair (`calculates_bip141_weight`)
- Rounding weight up to virtual size with `ceil(weight / 4)`
  (`rounds_weight_up_to_virtual_bytes`)
- Calculating a fee from virtual size and feerate, and rejecting overflow
  (`calculates_fee_from_feerate`)
- Reproducing the class comparison: 226 vB legacy vs. 141 vB SegWit at
  50 sat/vB, for fees of 11,300 sats vs. 7,050 sats and 4,250 sats saved
  (`reproduces_the_class_fee_comparison`)

## Explanation


Witness data isn't simply removed from size, and it isn't given one flat
discount across the whole transaction — it's weighted differently *within*
BIP141's formula, and that distinction matters.

BIP141 defines weight as `stripped_size * 4 + witness_size` (equivalently,
`stripped_size * 3 + total_size`, which is what this lab computes). Every
byte outside the witness — version, inputs, outputs, locktime — counts
four times toward weight. Every witness byte counts only once. Virtual size
is then `weight / 4`, rounded up.

The effect is that a transaction's non-witness bytes are still fully
"charged" at their normal weight — nothing is stripped away or ignored —
while witness bytes are effectively counted at one-quarter their actual
size. This produces roughly a 75% discount specifically on the witness
portion, not on the transaction as a whole. A P2WPKH transaction's savings
over P2PKH (141 vB vs. 226 vB in this lab, a difference of 85 vB) come
entirely from moving the signature and public key out of the 4x-weighted
base transaction and into the 1x-weighted witness — the non-witness parts
of both transactions are otherwise comparable in size.

If SegWit gave one flat discount to the whole transaction instead, a
transaction with little or no witness data (like a transaction spending
only P2WPKH inputs, but paying to another output with a smaller footprint)
would be discounted by the same proportion as one with heavy witness data
(like a multisig transaction with several signatures), which wouldn't
accurately reflect how much data each transaction actually requires nodes
to store and validate. Weighting witness bytes specifically — rather than
discounting the whole transaction — ties the fee incentive directly to the
part of the transaction that SegWit was designed to make cheaper: signature
and witness data.

