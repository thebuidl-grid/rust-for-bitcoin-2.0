# Lab 06 — Weight, virtual size, and fees

## Commands used

`cargo test --test lab_06`

## Terminal output

```bash
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.34s
     Running tests/lab_06.rs (target/debug/deps/lab_06-5143ef20effedfbe)

running 4 tests
test calculates_bip141_weight ... ok
test reproduces_the_class_fee_comparison ... ok
test calculates_fee_from_feerate ... ok
test rounds_weight_up_to_virtual_bytes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
``` 

## Evidence references

Code: src/labs/lab06_weight_fees.rs  
Test: tests/lab_06.rs

## Explanation

### Explain SegWit weight accounting without calling it a flat discount.

SegWit separates transaction data into non-witness data and witness data. Witness bytes contribute one weight unit each, while non-witness bytes contribute four. This reflects that witness data is stored outside the traditional transaction serialization and is excluded from the transaction ID calculation.

SegWit assigns different weights to different parts of a transaction, rather than applying a uniform percentage reduction to the entire transaction.