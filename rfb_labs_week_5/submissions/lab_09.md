# Lab 09 — BIP44 path decoding

## Commands used

`cargo test --test lab_09`

## Terminal output

All 4 tests passed. `m/44'/0'/2'/1/5` decoded to purpose 44, coin type 0, account 2, change 1, index 5; replacing the final index with 6 preserved the branch. Regtest derivation was deterministic.

## Evidence references

Evidence: terminal output from `cargo test --test lab_09`; tests cover path decoding, explanation, index replacement, and P2PKH derivation.

## Explanation

Purpose selects the wallet standard, coin type selects the currency namespace, and account separates users. Change 0 is the receiving chain and change 1 is the change chain. The final index selects one address on that branch.

