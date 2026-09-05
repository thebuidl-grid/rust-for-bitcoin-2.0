# Lab 06 — Weight, virtual size, and fees

## Commands used

```bash
cargo test --test lab_06
cargo fmt --check
```

## Terminal output

```text
$ cargo test --test lab_06
running 4 tests
test reproduces_the_class_fee_comparison ... ok
test calculates_bip141_weight ... ok
test calculates_fee_from_feerate ... ok
test rounds_weight_up_to_virtual_bytes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`cargo fmt --check` produced no diff.

## Evidence references

- Implementation: `src/labs/lab06_weight_fees.rs`
- Test suite: `tests/lab_06.rs`
- `transaction_weight(100, 200) == 500` (`100*3 + 200`), and `transaction_weight(201, 200)` errors
  because stripped size cannot exceed total size.
- `virtual_size(564) == 141` (exact division) and `virtual_size(565) == 142` (rounds up).
- `fee_sats(141, 50) == 7_050`; `fee_sats(u64::MAX, 2)` errors on overflow via `checked_mul`.
- `compare_fees(226, 141, 50)` reproduces the class numbers exactly: legacy fee `11,300` sats,
  SegWit fee `7,050` sats, savings `4,250` sats — matching the ~226 vB P2PKH vs. ~141 vB P2WPKH
  comparison from the Week 5 class.

## Explanation

SegWit does not give witness data a flat, whole-transaction discount — it gives it a *per-byte*
discount inside the weight formula, and that distinction matters. BIP141 defines
`weight = stripped_size * 3 + total_size`, where `stripped_size` is the transaction serialized
*without* any witness data and `total_size` includes it. Equivalently, weight counts every
non-witness byte 4 times and every witness byte only 1 time — a 4x discount that applies strictly
to the bytes that are actually witness data, not to the transaction as a whole. That's why you
can't just "remove" witness data to get the size for fee purposes: the base transaction (version,
inputs, outputs, locktime) is unaffected and still weighs full price, and two transactions with
identical stripped sizes but different witness sizes will have different weights, not a shared
flat reduction. `virtual_size = ceil(weight / 4)` then converts that weight back into a legacy-vB
unit so old fee-rate tooling (sat/vB) keeps working. In the class example, a P2WPKH input's
signature and pubkey move from ScriptSig (full weight) into the witness (discounted weight),
which is why the same economically-equivalent 1-input-2-output transaction costs ~141 vB instead
of ~226 vB, and why moving *more* data into the witness (e.g. multisig, Taproot script paths)
keeps paying off proportionally rather than hitting a fixed cap.

