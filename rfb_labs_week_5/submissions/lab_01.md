# Lab 01 — Address and network identification

## Commands used

`cargo test --test lab_01`

## Terminal output

All 4 tests passed. Prefixes were classified as P2PKH, P2SH, P2WPKH, or P2TR; valid addresses were parsed and checked against the requested network, and their scriptPubKeys were produced.

## Evidence references

Evidence: terminal output from `cargo test --test lab_01`; tests cover prefixes, scriptPubKeys, and wrong-network rejection.

## Explanation

Prefixes are only a human-readable clue. Complete validation must verify the checksum, encoding, payload length, address type, and network. The implementation parses the address and rejects a valid address supplied for the wrong network.

