# Lab 08 — BIP32 extended keys

## Commands used

```bash
cargo test --test lab_08
cargo fmt --check
bash grader/grade.sh
```

## Terminal output

```text
running 4 tests
test distinguishes_hardened_and_normal_paths ... ok
test derives_matching_extended_keys ... ok
test xpub_derives_a_normal_public_child ... ok
test creates_a_test_family_master_xpriv ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

## Evidence references

Automated unit tests in `tests/lab_08.rs` passing 4 checks that construct master extended private keys (`tprv...`), derive child keys at paths like `m/84'/1'/0'`, neuter xprivs to watch-only xpubs (`tpub...`), derive normal public child keys directly from parent xpubs, and detect hardened derivation steps.

## Explanation

BIP32 defines hierarchical deterministic wallets using extended keys, chain codes, and derivation paths:
- **Extended Key & Chain Code**: An extended key bundles a 256-bit key (private or public) with a 256-bit chain code. The chain code provides extra entropy to prevent an observer who knows one public key from guessing its sibling keys.
- **xpriv vs xpub**: An `xpriv` contains private key material and can derive both private and public child keys down any path. An `xpub` contains only public key material and can derive unhardened child public keys for watch-only wallets and payment gateways without exposing private keys.
- **Normal vs Hardened Derivation**:
  - *Normal derivation* (`index < 2^31`): Derives child public keys using `parent_pubkey` and `parent_chain_code`. This enables watch-only xpub derivation.
  - *Hardened derivation* (`index >= 2^31`, marked with `'` or `h`): Derives child keys using `parent_privkey` inside the HMAC-SHA512 hash. Because `parent_privkey` is required, an `xpub` cannot derive hardened child keys. Hardening prevents a security risk where leaking a single unhardened child private key along with a parent xpub would allow calculating the parent private key.
