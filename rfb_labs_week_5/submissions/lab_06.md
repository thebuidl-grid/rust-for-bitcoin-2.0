# Lab 06 — Weight, virtual size, and fees

## Commands used

```shell
test@pop-os:~/Desktop/rust/rust-for-bitcoin-2.0/rfb_labs_week_5$ cargo test --test lab_06
```

## Terminal output

```terminaloutput
running 4 tests
test calculates_bip141_weight ... ok
test calculates_fee_from_feerate ... ok
test reproduces_the_class_fee_comparison ... ok
test rounds_weight_up_to_virtual_bytes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

BIP141 weight: `transaction_weight(100, 200) = 500` (i.e. `100*3 + 200`).
Virtual size rounds weight up: `virtual_size(564) = 141`, `virtual_size(565) = 142`
(`ceil(weight/4)`). At 50 sat/vB: fee on 141 vB is 7050 sats.

Class comparison at 50 sat/vB:

| | vbytes | fee |
|---|---:|---:|
| Legacy P2PKH | 226 | 11,300 sats |
| Native P2WPKH | 141 | 7,050 sats |
| Savings | — | 4,250 sats |

## Evidence references

```
Code: src/labs/lab06_weight_fees.rs
Test: tests/lab_06.rs
```

## Explanation

BIP141 defines transaction **weight** as `stripped_size * 3 + total_size`, where
`stripped_size` is the size of the transaction **without** witness data and
`total_size` includes it. Witness data is counted once, while everything else (the
non-witness parts) is counted with a 4x multiplier. **Virtual size** is
`ceil(weight / 4)`, and the fee is `virtual_size * feerate`.

This is why SegWit savings are **not** simply "remove the witness, or apply one flat
discount to the whole transaction." The discount applies only to the witness part:
every non-witness byte still costs the full 4 weight units, so a 226-vB P2PKH
transaction is not halved to 113 vB. A native P2WPKH transaction reaches ~141 vB
because its public-key data moves into the (discounted) witness and the
scriptPubKey/scriptSig become much smaller — the measurement rule is what rewards
that move, and it does not give a blanket discount to legacy data.

This also matters for consensus: weight caps block size (the 4,000,000 weight-unit
limit) while the weight-based vsize is what fee estimators use, so security and
fee-market incentives stay aligned.
