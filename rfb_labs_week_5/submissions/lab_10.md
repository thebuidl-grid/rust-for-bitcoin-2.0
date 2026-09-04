# Lab 10 — Deterministic recovery across address families

## Commands used

`cargo test --test lab_10` using the public disposable test mnemonic.

## Terminal output

All 4 tests passed. The regtest families had expected prefixes: BIP44 P2PKH `m...` or `n...`, BIP49 wrapped SegWit `2...`, and BIP84 native SegWit `bcrt1q...`. Identical inputs reproduced the same address; changing the index changed it.

## Evidence references

Evidence: terminal output from `cargo test --test lab_10`; only public disposable data was used, and wallet secrets were not recorded.

## Explanation

Recovery is deterministic because the mnemonic, passphrase, network, derivation path, and address encoding rules determine each child key and address. BIP44, BIP49, and BIP84 use different purpose branches and output formats, producing distinct but reproducible address families.

