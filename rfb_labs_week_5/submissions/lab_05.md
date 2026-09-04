# Lab 05 — Address compatibility map

## Commands used

`cargo test --test lab_05`

## Terminal output

All 4 tests passed. A Base58Check-only P2SH-era sender reports `p2pkh=true`, `p2sh_p2wpkh=true`, `p2wpkh=false`, and `p2tr=false`.

## Evidence references

Evidence: terminal output from `cargo test --test lab_05`; tests cover capability mapping, preference ordering, and encoding names.

## Explanation

A P2SH-era wallet may understand Base58Check and the P2SH version byte, so it can decode `3...`. A `bc1q...` destination uses Bech32 and a SegWit witness program, requiring newer support.

