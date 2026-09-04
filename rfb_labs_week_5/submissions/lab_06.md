# Lab 06 — Weight, virtual size, and fees

## Commands used

```
cargo test --test lab_06 -- --nocapture
cargo fmt --check
```

Implementation: `src/labs/lab06_weight_fees.rs` — `transaction_weight`,
`virtual_size`, `fee_sats`, `compare_fees`.

## Terminal output

```
running 4 tests
test reproduces_the_class_fee_comparison ... ok
test rounds_weight_up_to_virtual_bytes ... ok
test calculates_fee_from_feerate ... ok
test calculates_bip141_weight ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Observed behaviour:
- `transaction_weight(100, 200)` = `500` (= 100*3 + 200); `transaction_weight(201, 200)` errors
  because total < stripped.
- `virtual_size(564)` = `141`, `virtual_size(565)` = `142` (ceil division by 4).
- `fee_sats(141, 50)` = `7050`; `fee_sats(u64::MAX, 2)` errors on overflow.
- `compare_fees(226, 141, 50)` = legacy `11300`, segwit `7050`, savings `4250`.

## Evidence references

- `src/labs/lab06_weight_fees.rs` — checked arithmetic, `div_ceil`, saturating savings.
- `tests/lab_06.rs` — the BIP141 weight identity and the class fee comparison numbers.
- Terminal block above from `cargo test --test lab_06`.

## Explanation

BIP141 does not give SegWit a flat percentage discount. It redefines the size unit.
Weight = (bytes that are *not* witness data) x 4 + (witness bytes) x 1, which is
equal to `stripped_size * 3 + total_size` (stripped counts non-witness bytes once,
total counts everything once, so non-witness bytes end up weighted 4 and witness
bytes 1). Virtual size is `ceil(weight / 4)`, so a byte in the non-witness part
still costs a full vbyte while a witness byte costs a quarter. The "discount" is
therefore an emergent property: a transaction that moves data (signatures, pubkeys)
into the witness shrinks its vbyte count and pays `vbytes * feerate` on the smaller
number. A legacy 226-vbyte spend and a native-SegWit 141-vbyte spend at 50 sat/vB
cost 11 300 vs 7 050 sats — the 4 250-sat gap comes entirely from where the
unlocking bytes are accounted, not from a special rate.
