# Lab 02 — Legacy P2PKH

## Commands used

`cargo test --test lab_02`

## Terminal output

All 4 tests passed. HASH160 is committed in the standard P2PKH script, and the ScriptSig model contains signature then public key.

## Evidence references

Evidence: terminal output from `cargo test --test lab_02`; tests verify address derivation, script construction, HASH160, and ScriptSig placement.

## Explanation

P2PKH locks coins to a HASH160 commitment of a public key. Spending supplies a signature and matching public key; script execution checks the hash and then verifies the signature.

