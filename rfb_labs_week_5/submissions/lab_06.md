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
test rounds_weight_up_to_virtual_bytes ... ok
test reproduces_the_class_fee_comparison ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

All four public tests in `tests/lab_06.rs` pass against `src/labs/lab06_weight_fees.rs`:
`transaction_weight(100, 200) == 500` (`100*3 + 200`) and rejects an inconsistent
`stripped_size > total_size`; `virtual_size` rounds `564` → `141` and `565` → `142`
(`ceil(weight / 4)`); `fee_sats(141, 50) == 7_050` and rejects a multiplication that would
overflow `u64`; `compare_fees(226, 141, 50)` reproduces the class numbers exactly: legacy
11,300 sats, SegWit 7,050 sats, 4,250 sats saved.

## Explanation

Witness data isn't simply deleted from the size calculation, nor is it given one flat discount
applied to the whole transaction — BIP141 gives it a *different weight per byte* than the rest of
the transaction, and that distinction is deliberate. `transaction_weight` computes
`stripped_size * 3 + total_size`, where `stripped_size` is the transaction *without* any witness
data and `total_size` is the full serialized transaction *with* witness data included. Because
`total_size - stripped_size` is exactly the witness byte count, this formula is equivalent to
`stripped_size * 4 + witness_size * 1` — every non-witness byte counts 4x, every witness byte
counts 1x. Only after that weighted sum is computed do we take `virtual_size = ceil(weight / 4)`
to get a single "virtual byte" figure comparable to legacy transaction sizes.

If witness data were simply *removed* from the size entirely, two transactions with wildly
different witness sizes (e.g. a single signature vs. a large multisig witness) would look
identical for fee purposes despite consuming different amounts of block space and validation
work — that would let SegWit transactions with heavy witnesses free-ride on block space. If
instead a single flat discount were applied to the *whole* transaction, transactions with little
or no witness data (which don't need much of a discount) would be discounted just as much as
those that are almost entirely witness bytes, again mispricing block space. Weighting per-byte by
witness-vs-non-witness is what lets `compare_fees(226, 141, 50)` reproduce the real class number:
a P2WPKH spend's witness portion is priced at 1/4 the weight of its non-witness portion, which is
exactly why 141 vB costs 7,050 sats instead of scaling directly with the transaction's raw byte
count the way the 226 vB legacy transaction does.
