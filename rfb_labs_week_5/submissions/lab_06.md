# Lab 06 — Weight, virtual size, and fees

## Commands used

```bash
cargo test --test lab_06
cargo fmt --check
bash grader/grade.sh
```

## Terminal output

```text
running 4 tests
test calculates_bip141_weight ... ok
test calculates_fee_from_feerate ... ok
test reproduces_the_class_fee_comparison ... ok
test rounds_weight_up_to_virtual_bytes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

Unit tests in `tests/lab_06.rs` verifying BIP141 weight calculations (`weight = stripped_size * 3 + total_size`), virtual size rounding (`vbytes = ceil(weight / 4)`), fee multiplication (`fee = vbytes * feerate`), and reproducing the class comparison showing 11,300 sats for P2PKH (226 vB) vs 7,050 sats for P2WPKH (141 vB) at 50 sat/vB (4,250 sats savings).

## Explanation

SegWit weight accounting under BIP141 does not simply delete witness data or grant a flat whole-transaction fee discount. Instead, it introduces Weight Units (WU) as the fundamental unit of block capacity limit (4,000,000 WU per block). Non-witness bytes (version, input/output counts, outpoints, scriptSigs, scriptPubKeys, locktime) are assigned 4 WUs per byte, while witness stack items (signatures, public keys) are assigned 1 WU per byte. This formula `weight = (stripped_bytes * 3) + total_bytes` leads to a virtual size `vbytes = ceil(weight / 4)`. The 4:1 weight ratio incentivizes clean UTXO management because witness bytes do not permanently bloat the active in-memory UTXO set after transaction verification.
