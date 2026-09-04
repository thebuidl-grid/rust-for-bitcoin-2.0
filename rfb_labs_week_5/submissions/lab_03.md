# Lab 03 — P2SH 2-of-3 multisig

## Commands used

`cargo test --test lab_03`

## Terminal output

All 4 tests passed. The redeemScript is a canonical 2-of-3 multisig script; the P2SH address and outer scriptPubKey commit to it.

## Evidence references

Evidence: terminal output from `cargo test --test lab_03`; tests verify the redeemScript, address, outer lock, and report.

## Explanation

The outer P2SH check requires a redeemScript whose HASH160 matches the script hash. The revealed script then requires two valid signatures for the three committed public keys.

