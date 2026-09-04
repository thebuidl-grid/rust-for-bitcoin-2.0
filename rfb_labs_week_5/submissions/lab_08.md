# Lab 08 — BIP32 extended keys and hardened derivation

## Commands used

```
cargo test --test lab_08 -- --nocapture
cargo fmt --check
```

Implementation: `src/labs/lab08_bip32.rs` — `master_xpriv`, `derive_extended_keys`,
`derive_normal_child_xpub`, `path_contains_hardened_step`. HD helpers in
`src/labs/common.rs`.

## Terminal output

```
running 4 tests
test distinguishes_hardened_and_normal_paths ... ok
test derives_matching_extended_keys ... ok
test xpub_derives_a_normal_public_child ... ok
test creates_a_test_family_master_xpriv ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Observed behaviour with the public test mnemonic, empty passphrase, `Network::Regtest`:
- `master_xpriv` starts with `tprv` and is byte-for-byte identical across repeated calls.
- `derive_extended_keys(m, "", "m/84'/1'/0'")` returns `derivation_path == "m/84'/1'/0'"`,
  an `xpriv` starting `tprv`, and an `xpub` starting `tpub`.
- From the `m/84'/1'/0'/0` xpub, `derive_normal_child_xpub(.., 7)` returns a different
  `tpub` string (a non-hardened child).
- `path_contains_hardened_step("m/44'/0'/0'/0/0")` = true, `"m/0/1/2"` = false,
  `"not/a/path"` errors.

## Evidence references

- `src/labs/lab08_bip32.rs` — `Xpub::from_priv`, `Xpub::ckd_pub`, `ChildNumber::is_hardened`.
- `src/labs/common.rs` — `Xpriv::new_master` and `derive_priv` over a parsed `DerivationPath`.
- `tests/lab_08.rs` — the `tprv`/`tpub` prefixes and the hardened/normal path distinction.

## Explanation

An **xpriv** bundles a 32-byte private key with a 32-byte **chain code**; the
**xpub** replaces the private key with its public point but keeps the *same* chain
code. Child derivation feeds `HMAC-SHA512(key = chain code, data = parent key
material || index)`; the left half tweaks the key and the right half becomes the
child chain code. For a **normal** (non-hardened, index `< 2^31`) child the "key
material" is the parent *public* key, so an xpub alone can derive child xpubs —
handy for watch-only wallets and gap-limit scanning. For a **hardened** child
(index `>= 2^31`, written `'`) the parent *private* key is used instead, so an xpub
cannot compute it. Hardening is a firewall: without it, a leaked xpub plus any one
child private key lets an attacker recover the parent private key and every
sibling. BIP44-style paths therefore harden the account-level and above
(`purpose'/coin'/account'`) and leave the change/index levels normal so an account
xpub can still be exported safely.
