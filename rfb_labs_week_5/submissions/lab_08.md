# Lab 08 — BIP32 extended keys

## Commands used

```shell
test@pop-os:~/Desktop/rust/rust-for-bitcoin-2.0/rfb_labs_week_5$ cargo test --test lab_08
```

Only the disposable class data and disposable test keys were used.

## Terminal output

```terminaloutput
running 4 tests
test distinguishes_hardened_and_normal_paths ... ok
test derives_matching_extended_keys ... ok
test xpub_derives_a_normal_public_child ... ok
test creates_a_test_family_master_xpriv ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The regtest master xpriv is deterministic (same inputs → same `tprv...`). Deriving
at `m/84'/1'/0'` yields a matching xpriv (`tprv...`) and neutered xpub (`tpub...`).
`derive_normal_child_xpub` derives a non-hardened public child from a parent xpub
(regtest `tpub...`). `path_contains_hardened_step` returns true for `m/44'/0'/0'/0/0`
and false for `m/0/1/2`.
(Full extended keys are redacted to their prefixes here, per the safety guidance.)

## Evidence references

```
Code: src/labs/lab08_bip32.rs
Test: tests/lab_08.rs
```

## Explanation

BIP32 represents a key as an **extended key**: a private/public key plus a **chain
code**. The chain code is 32 bytes of entropy mixed into every child derivation; it
is what keeps different paths from trivially deriving each other and provides the
HMAC key material for `CKD`. Without it, the tree would be predictable and parent
keys could be recomputed from children.

**xpriv** is the extended private key. It can derive any child (normal or hardened)
and, from a private child, its public counterpart. Because it grants full control
(any descendant key), it is the secret material.

**xpub** is the extended public key, obtained by "neutering" an xpriv
(`Xpub::from_priv(secp, &child)`). It carries the same chain code and can derive
**non-hardened** public children and addresses, which makes it ideal for watch-only
wallets and "xpub for the accountant" use. An xpub reveals no private key and cannot
sign.

**Hardened versus normal derivation.** A normal child (`ChildNumber::Normal`) is
derived from the parent's *public* data, so an xpub can compute it. A hardened child
(`ChildNumber::Hardened`) is derived only from the parent's *private* key material.
This is precisely why a hardened child **cannot** be derived from a parent xpub — the
xpub has no private key to feed the hardened derivation. Hardening therefore
prevents an attacker who obtains a parent xpub from enumerating the child of a
hardened step; it also protects descendants: even if a normal child xpub leaks, an
attacker cannot climb back up to the parent's hardened branch.
