# Lab 06 — Weight, virtual size, and fees

## Commands used

```
cargo test --test lab_06 -- --nocapture
```

## Terminal output

```
running 4 tests
test calculates_bip141_weight ... ok
test rounds_weight_up_to_virtual_bytes ... ok
test reproduces_the_class_fee_comparison ... ok
test calculates_fee_from_feerate ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

```
transaction_weight(100, 200)      = 500        (3 * stripped_size + total_size)
transaction_weight(201, 200)      = Err(...)    (total_size can't be smaller than stripped_size)
virtual_size(564)                 = 141
virtual_size(565)                 = 142
fee_sats(141, 50)                 = 7_050
fee_sats(u64::MAX, 2)             = Err(...)    (overflow rejected)
compare_fees(226, 141, 50)        = FeeComparison {
                                       legacy_vbytes: 226, segwit_vbytes: 141,
                                       legacy_fee_sats: 11_300, segwit_fee_sats: 7_050,
                                       savings_sats: 4_250,
                                     }
```

This reproduces the class comparison: a P2PKH spend at ~226 vB costs 11,300 sats at
50 sat/vB, while an equivalent P2WPKH spend at ~141 vB costs 7,050 sats — a savings of
4,250 sats for the same economic transaction.

## Explanation

BIP141 weight is `3 * stripped_size + total_size`, which is equivalent to counting every
non-witness byte 4 times and every witness byte once (`total_size = stripped_size +
witness_size`, so `weight = 4 * stripped_size + witness_size`). This is not a flat
whole-transaction discount because the discount only applies to the witness portion —
inputs, outputs, locktime, and version bytes are still charged at full weight regardless
of transaction type. A P2WPKH transaction is cheaper specifically because its signature
and public key move out of the base (4x-weighted) transaction and into the witness
(1x-weighted) section, not because SegWit transactions get some percentage knocked off
their total size. Two transactions with the same total byte count but different
witness/base splits will have different weights and different fees at the same feerate;
you cannot compute the discount by looking at total_size alone, only by knowing how many
of those bytes are witness bytes.
