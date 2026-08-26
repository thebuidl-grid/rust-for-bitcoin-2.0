# Lab 06 — Weight, virtual size, and fees

## Commands used

```bash
cargo test --test lab_06 -- --nocapture
cargo run --example labs_demo
```

## Terminal output

```text
$ cargo test --test lab_06 -- --nocapture
running 4 tests
test calculates_fee_from_feerate ... ok
test calculates_bip141_weight ... ok
test reproduces_the_class_fee_comparison ... ok
test rounds_weight_up_to_virtual_bytes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

```text
$ cargo run --example labs_demo   (Lab 06 section)
transaction_weight(100, 200) = 500
virtual_size(564) = 141
virtual_size(565) = 142
fee_sats(141, 50) = Ok(7050)
compare_fees(226, 141, 50) = Ok(FeeComparison {
    legacy_vbytes: 226, segwit_vbytes: 141,
    legacy_fee_sats: 11300, segwit_fee_sats: 7050, savings_sats: 4250 })
```

## Evidence references

- Implementation: [`src/labs/lab06_weight_fees.rs`](../src/labs/lab06_weight_fees.rs)
- Public test suite: [`tests/lab_06.rs`](../tests/lab_06.rs) — 4/4 passing, logged in
  [`grading/logs/lab_06.log`](../grading/logs/lab_06.log).
- Reproduces the class figures exactly: 226 vB legacy P2PKH vs. 141 vB native P2WPKH at
  50 sat/vB → 11,300 sats vs. 7,050 sats, a 4,250-sat saving (≈37.6%).

## Explanation

BIP141 defines `weight = stripped_size * 3 + total_size`, where `stripped_size` is the
transaction serialized *without* the witness (marker, flag, and all witness stacks
removed) and `total_size` includes the witness. Rearranged, this is equivalent to
`weight = 3 * stripped_size + (stripped_size + witness_size) = 4 * stripped_size +
witness_size`. That form makes the accounting explicit: every **non-witness** byte
(the parts every node, including pre-SegWit ones, must store and validate — outpoints,
scriptSigs, amounts, locktimes) is charged the full 4 weight units, while every
**witness** byte is charged only 1 weight unit.

This is why it is wrong to describe SegWit's saving as removing witness data or as a
single flat discount applied to the whole transaction: nothing is removed (the witness
is still fully validated and still occupies real disk/bandwidth), and the discount is
not applied transaction-wide — it applies *per byte*, and only to the subset of bytes
that live in the witness. A P2WPKH input still has an (empty) ScriptSig field costing 4
weight units per byte just like a P2PKH input's ScriptSig would; the difference is that
P2WPKH's signature and pubkey have moved out of that 4x-weighted ScriptSig and into the
1x-weighted witness, while its non-witness footprint (outpoint, sequence, the tiny
`OP_0 <hash>` scriptPubKey) stays roughly the same size as P2PKH's.

`virtual_size = ceil(weight / 4)` translates weight units back into a size measured in
the same units miners are used to (bytes), by definition making a legacy, all-non-witness
transaction's vsize equal its literal byte size, while a SegWit transaction's vsize is
smaller than its literal total byte size. Fees are then charged per virtual byte
(`fee_sats = vbytes * feerate_sat_vb`), so every witness byte a transaction can shift
out of ScriptSig and into the witness directly reduces the fee owed at a given feerate
— which is exactly the 4,250-sat gap reproduced above.
