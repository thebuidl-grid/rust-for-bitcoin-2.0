# Lab 06 — Weight, virtual size, and fees

## Commands used

```
cargo test --test lab_06 -- --nocapture
```

Ad-hoc check reproducing the class comparison at 50 sat/vB:

```rust
lab06_weight_fees::compare_fees(226, 141, 50)
```

## Terminal output

```
running 4 tests
test calculates_bip141_weight ... ok
test calculates_fee_from_feerate ... ok
test reproduces_the_class_fee_comparison ... ok
test rounds_weight_up_to_virtual_bytes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```
transaction_weight(100, 200) = 500
virtual_size(564) = 141
virtual_size(565) = 142
fee_sats(141, 50) = 7050
FeeComparison { legacy_vbytes: 226, segwit_vbytes: 141, legacy_fee_sats: 11300, segwit_fee_sats: 7050, savings_sats: 4250 }
```

`transaction_weight(201, 200)` and `fee_sats(u64::MAX, 2)` both return `Err`, as
required by `calculates_bip141_weight` and `calculates_fee_from_feerate`.

## Evidence references

- `cargo test --test lab_06` output above.
- Source: `src/labs/lab06_weight_fees.rs`.
- Test suite: `tests/lab_06.rs`.

## Explanation

BIP141 weight is `stripped_size * 3 + total_size`, not `total_size` minus some flat
percentage for witness data. The `stripped_size` (the transaction with the witness
field removed) is counted three extra times and the full `total_size` (base data plus
witness data) is counted once, which is mathematically equivalent to weighting
non-witness bytes at 4 and witness bytes at 1. That distinction matters: two
transactions with the same total byte count but different splits between base data
and witness data get different weights, because only the witness portion gets the
discount. A single flat discount applied to the whole transaction would ignore how
much of that transaction is actually witness data. Virtual size then converts weight
back into a legacy-comparable unit by dividing by 4 and rounding up, which is what
lets a feerate quoted in sat/vB apply uniformly to both legacy and SegWit
transactions. The 226 vB (P2PKH) versus 141 vB (P2WPKH) comparison at 50 sat/vB
reproduces the roughly 4,250-satoshi saving from moving the same economic transaction
onto native SegWit.
