# Lab 04 — Native P2WPKH

## Commands used

`cargo test --test lab_04`

## Terminal output

All 4 tests passed. The address is native version-0 Bech32, the witness program is a 20-byte public-key hash, ScriptSig is empty, and witness items are signature then public key.

## Evidence references

Evidence: terminal output from `cargo test --test lab_04`; tests verify the address, witness lock, program, and witness placement.

## Explanation

Native SegWit commits the witness program directly in the scriptPubKey. Unlocking data belongs in the witness field, so ScriptSig remains empty.

