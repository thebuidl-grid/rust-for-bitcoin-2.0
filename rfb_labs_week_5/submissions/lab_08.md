# Lab 08 — BIP32 extended keys

## Commands used

`cargo test --test lab_08` using the public disposable test mnemonic.

## Terminal output

All 4 tests passed. Regtest private keys used `tprv` prefixes and public keys used `tpub` prefixes. Normal public derivation produced a different child xpub, and hardened path components were detected.

## Evidence references

Evidence: terminal output from `cargo test --test lab_08`. Extended private key values were intentionally not copied into this submission.

## Explanation

An xpriv carries private key material plus metadata and chain code. An xpub carries the public key and chain code, allowing normal child derivation without private material. Hardened derivation requires the parent private key; normal derivation can be performed from an xpub.

