# Lab 08 — BIP32 extended keys

## Commands used

```bash
cargo test --test lab_08 -- --nocapture
cargo fmt --check
cargo clippy --all-targets
```

Implementation lives in `src/labs/lab08_bip32.rs`: `master_xpriv`,
`derive_extended_keys`, `derive_normal_child_xpub`, and
`path_contains_hardened_step`, all built on the published class mnemonic.

## Terminal output

```
running 4 tests
test distinguishes_hardened_and_normal_paths ... ok
test derives_matching_extended_keys ... ok
test xpub_derives_a_normal_public_child ... ok
test creates_a_test_family_master_xpriv ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

## Evidence references

- `src/labs/lab08_bip32.rs` — master key creation on Regtest (`tprv...` /
  `tpub...` prefixes, shared with testnet), full-path derivation to an
  `xpriv`/`xpub` pair, and non-hardened `xpub`-only child derivation.
- `tests/lab_08.rs` — `creates_a_test_family_master_xpriv` checks the same
  mnemonic/passphrase/network always reproduces the identical `tprv...`;
  `xpub_derives_a_normal_public_child` derives child index 7 from a parent
  `tpub` with no private key material involved anywhere in that call;
  `distinguishes_hardened_and_normal_paths` checks `m/44'/0'/0'/0/0` is
  reported as hardened, `m/0/1/2` is not, and `not/a/path` errors.
- `bash grader/grade.sh` recorded `08 | 4/4 | 4 | ...` for this lab.

## Explanation

The chain code is a 32-byte value carried alongside every extended key
precisely so that "the same 256-bit number, twice" does not derive the same
child tree twice. Both an `xpriv` and its neutered `xpub` combine their key
material with the chain code, an index, and (for private derivation) the
parent's public key inside HMAC-SHA512 to produce each child's key and chain
code, so the chain code is what makes the resulting tree of billions of keys
deterministic yet unpredictable from the index alone — without it, siblings
derived from a bare public/private key pair would collide or be trivially
related.

An `xpub` is deliberately "watch-only" because BIP32 extended public keys
carry no private key material at all, only the public point and chain code —
`derive_normal_child_xpub` derives address after address from `parent.xpub`
alone and never touches a private key, which is exactly why a server or
watch-only wallet can hand out `xpub`s to monitor incoming payments across an
entire account without being able to spend anything it receives. That
convenience has a sharp edge, though: hardened derivation
(BIP32 indices ≥ 2³¹, `path_contains_hardened_step` flags them) folds the
*parent private key* into the HMAC-SHA512 input instead of the parent public
key, specifically so that leaking a child private key plus the parent chain
code can never be combined to recover the parent's own private key — a real
risk with non-hardened derivation. Because that HMAC step requires the
private key as input, and an `xpub` never has one, hardened children simply
cannot be computed from `derive_normal_child_xpub`'s public-key-only inputs at
all; that operation only exists in the `Xpriv` path in `derive_extended_keys`.
This is also why BIP44/49/84 always harden purpose, coin type, and account —
those levels stay hardened-only so the corresponding account-level `xpub` can
still be shared for watch-only use without exposing the parent's private key.
