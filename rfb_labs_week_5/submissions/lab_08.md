# Lab 08 — BIP32 extended keys

## Commands used

I formatted the source and ran the complete BIP32 test suite with the public BIP39
test mnemonic and regtest network parameters:

```bash
cargo fmt
cargo test --test lab_08 creates_a_test_family_master_xpriv
cargo test --test lab_08 derives_matching_extended_keys
cargo test --test lab_08 xpub_derives_a_normal_public_child
cargo test --test lab_08 distinguishes_hardened_and_normal_paths
cargo test --test lab_08
```

## Terminal output

Only safe prefixes and structural observations were recorded:

```text
regtest master extended private key prefix: tprv
derived path: m/84'/1'/0'
derived extended private key prefix: tprv
derived extended public key prefix: tpub
normal child index 7 derived from parent xpub: success
hardened step detected in m/44'/0'/0'/0/0: true
hardened step detected in m/0/1/2: false

test result: ok. 4 passed; 0 failed
```

## Evidence references

- Implementation: `src/labs/lab08_bip32.rs`
- Public tests: `tests/lab_08.rs`
- The implementation validates the mnemonic and path, creates a deterministic
  network-specific master xpriv, derives the requested child xpriv, and neuters it
  to an xpub.
- Full xpriv values are intentionally omitted because an xpriv controls every
  private child beneath it.

## Explanation

A BIP32 extended private key contains a private key plus a chain code and derivation
metadata. It can derive both private and public descendants. An extended public key
contains the matching public key, chain code, and metadata; it supports watch-only
wallets that can generate normal child public keys and addresses without possessing
private signing material.

The chain code supplies additional deterministic input for child derivation. It is
not itself a signing key, but an ordinary public or private key without its chain
code cannot reproduce the same BIP32 subtree. Normal derivation allows the same
public child to be derived from a parent xpub or parent xpriv. Hardened derivation
incorporates private key material, so a hardened child cannot be derived from an
xpub. Hardened path components are marked with an apostrophe, such as `84'`.
