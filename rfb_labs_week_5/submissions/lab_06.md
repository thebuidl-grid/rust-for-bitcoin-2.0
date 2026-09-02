# Lab 06 — Weight, virtual size, and fees

## Commands used

```bash
cargo test --test lab_06 -- --nocapture
cargo fmt --check
cargo clippy --all-targets
```

Implementation lives in `src/labs/lab06_weight_fees.rs`: `transaction_weight`,
`virtual_size`, `fee_sats`, and `compare_fees`.

## Terminal output

```
running 4 tests
test calculates_fee_from_feerate ... ok
test calculates_bip141_weight ... ok
test reproduces_the_class_fee_comparison ... ok
test rounds_weight_up_to_virtual_bytes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`reproduces_the_class_fee_comparison` reproduces the class numbers directly:
at 50 sat/vB, 226 vB legacy costs 11,300 sats and 141 vB SegWit costs
7,050 sats, a 4,250 sat saving — matching the README's "≈226 vB for P2PKH
versus 141 vB for P2WPKH" comparison.

## Evidence references

- `src/labs/lab06_weight_fees.rs` — `weight = stripped_size * 3 + total_size`
  (BIP141), `vbytes = ceil(weight / 4)`, and overflow-checked fee
  multiplication.
- `tests/lab_06.rs` — `calculates_bip141_weight` also checks that
  `stripped_size > total_size` is rejected as an invalid input;
  `calculates_fee_from_feerate` checks that `fee_sats(u64::MAX, 2)` errors
  instead of silently wrapping.
- `bash grader/grade.sh` recorded `06 | 4/4 | 4 | ...` for this lab.

## Explanation

Witness data isn't stripped from the transaction and it isn't given one flat
discount on the whole transaction — BIP141 weights the *base* (non-witness)
serialization and the witness serialization differently, and only virtual
size, not weight, is what "vbytes" and feerates are ever expressed in.
Concretely, `transaction_weight` implements
`weight = stripped_size * 3 + total_size`, where `stripped_size` is the
serialized size with the witness field removed and `total_size` includes it.
Algebraically that is `weight = 4 * stripped_size + witness_size`, i.e. every
non-witness byte counts 4x toward weight while every witness byte counts only
1x. That is a 75% *relative* discount applied per-byte to witness data
specifically, not a discount subtracted once from the transaction total —
inputs with large witnesses (many signatures) save much more than inputs with
none, so the effective discount is per-input, not a fixed transaction-wide
number, and a transaction with no witness data at all (an all-legacy tx) gets
no discount whatsoever, exactly as `virtual_size` shows: for a legacy tx,
`stripped_size == total_size`, so weight is just `4 * size` and vbytes equals
size — no reduction. Only once weight is computed from these two differently
weighted components is it rounded up to virtual bytes (`ceil(weight / 4)`),
and the feerate a wallet chooses is then applied to *that* number, which is
how the class's ≈226 vB legacy vs. ≈141 vB SegWit comparison — and the same
4,250 sat saving reproduced above — arise from otherwise similar-looking
single-input, single-output transactions.
