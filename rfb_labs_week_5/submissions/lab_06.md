# Lab 06 — Weight, virtual size, and fees

## Commands used

```bash
cargo test --test lab_06 -- --nocapture
```

## Terminal output

```
running 4 tests
test calculates_bip141_weight ... ok
test calculates_fee_from_feerate ... ok
test reproduces_the_class_fee_comparison ... ok
test rounds_weight_up_to_virtual_bytes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Evidence references

`src/labs/lab06_weight_fees.rs`:

- `transaction_weight(stripped, total) = stripped * 3 + total`, the BIP141 formula, and
  rejects `total_size < stripped_size` as an impossible input (total always includes at
  least what stripped counts).
- `virtual_size(weight) = weight.div_ceil(4)` — rounds up per BIP141.
- `fee_sats` uses `checked_mul` and returns an error on overflow rather than wrapping.
- `compare_fees(226, 141, 50)` reproduces the class numbers: legacy fee = 226 × 50 =
  11,300 sats, SegWit fee = 141 × 50 = 7,050 sats, savings = 4,250 sats — matching the
  ~226 vB P2PKH vs ~141 vB P2WPKH comparison from class.

## Explanation

The reason it's `stripped_size * 3 + total_size` and not some flat multiplier on the
whole transaction: BIP141 needs the base transaction — version, inputs, outputs,
locktime, everything a pre-SegWit node can actually parse — priced exactly the same
as it always was. Only the new witness field gets a discount, because that's the part
old nodes never see and never validate.

Work through the formula and that's what falls out. `stripped_size` (non-witness)
gets counted three extra times on top of the one time it's already inside
`total_size`, landing at weight ×4 — full price, unchanged from pre-SegWit. Witness
bytes appear once, only in `total_size`, so they land at ×1 — a quarter cost per byte.
Dividing by 4 to get vbytes is just so fee math still reads in familiar units; the
real mechanism is that 4:1 split, and it's deliberate — it's what stops SegWit from
quietly making the base transaction cheaper too, and stops anyone from stuffing
arbitrary data into a block "for free" outside the witness.
