# Lab 06 — Weight, virtual size, and fees

**Author:** [Christopher Dominic Eze](https://github.com/Christopherdominic)

## Commands used

```bash
cargo test --test lab_06
cargo run --example evidence   # scratch script, deleted after copying the output below
```

## Terminal output

```
running 4 tests
test calculates_fee_from_feerate ... ok
test rounds_weight_up_to_virtual_bytes ... ok
test reproduces_the_class_fee_comparison ... ok
test calculates_bip141_weight ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Class comparison at 50 sat/vB:

```
FeeComparison { legacy_vbytes: 226, segwit_vbytes: 141, legacy_fee_sats: 11300, segwit_fee_sats: 7050, savings_sats: 4250 }
```

## Evidence references

- `src/labs/lab06_weight_fees.rs` — `transaction_weight`, `virtual_size`,
  `fee_sats`, `compare_fees`.
- `tests/lab_06.rs::calculates_bip141_weight` — `transaction_weight(100, 200) == 500`
  (`100*3 + 200`), and `transaction_weight(201, 200)` is an error because the stripped
  size can't be bigger than the total size.
- `tests/lab_06.rs::reproduces_the_class_fee_comparison` — 226 vB legacy vs. 141 vB
  SegWit at 50 sat/vB reproduces the exact numbers above, matching the class example.

## Explanation

BIP141 weight is `stripped_size * 3 + total_size`, where `stripped_size` is the
transaction serialized *without* any witness data, and `total_size` is the full
serialization *with* witness data included. Rearranged, that's the same as
`stripped_size * 4 + witness_size` — because `total_size = stripped_size + witness_size`,
so `stripped*3 + (stripped + witness) = stripped*4 + witness`. That form makes the
intent obvious: every non-witness byte counts 4x toward weight, every witness byte
counts 1x. Virtual size is just weight divided by 4 (rounded up), which is what
mempools and fee estimators actually use, so that a legacy transaction's vsize equals
its literal byte size (since it has no witness data to discount) and a SegWit
transaction's vsize comes out lower than its literal byte size.

The reason this isn't "witness data is free" or "SegWit transactions get some flat
discount" is that the discount is per-byte and only applies to the witness portion.
A P2WPKH input still has to pay full weight for its non-witness bytes (the outpoint,
sequence, the `OP_0 <hash>` scriptPubKey it's spending, etc.) — those are 4 weight
units each, same as a legacy input's equivalent bytes. It's specifically the
signature and pubkey that moved into the witness that get the 4x discount, because
only *those* bytes are weighted at 1 instead of 4. That's why the 226 vB vs. 141 vB comparison isn't a round "SegWit is X% smaller"
statement — it's the direct consequence of the signature and pubkey for a typical
1-input-2-output transaction moving from a 4x-weighted location to a 1x-weighted one,
while every other byte in the transaction (outpoints, amounts, scriptPubKeys,
sequence numbers, locktime) stays weighted exactly the same as it always was.
