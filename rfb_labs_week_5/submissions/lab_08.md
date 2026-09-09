# Lab 08 — BIP32 extended keys

## Commands used

```bash
cargo test --test lab_08
bash grader/grade.sh
```

## Terminal output

running 4 tests

test distinguishes_hardened_and_normal_paths ... ok

test derives_matching_extended_keys ... ok

test xpub_derives_a_normal_public_child ... ok

test creates_a_test_family_master_xpriv ... ok

## Evidence references

ll four public tests in `tests/lab_08.rs` pass, using only the published
public BIP39 test mnemonic with an empty passphrase:
- Master xpriv creation is deterministic and correctly prefixed for regtest
  (`creates_a_test_family_master_xpriv`)
- A full path (`m/84'/1'/0'`) derives a matching xpriv/xpub pair, both
  correctly prefixed (`derives_matching_extended_keys`)
- A normal child index derives correctly from an xpub alone, with no
  private key material involved, and differs from its parent
  (`xpub_derives_a_normal_public_child`)
- Hardened and normal paths are correctly distinguished, and a malformed
  path is rejected (`distinguishes_hardened_and_normal_paths`)

## Explanation

An **xpriv** (extended private key) bundles a private key together with a
**chain code** — 32 extra bytes of entropy that aren't secret on their own,
but are mixed into every child derivation alongside the parent key. The
chain code's purpose is to make each child key derivation depend on more
than just the parent key and an index; without it, sibling keys derived at
different indices from the same parent would be related in a way that
could leak information about each other. The chain code is what makes each
branch of the tree cryptographically independent while still being
deterministically reproducible from the same seed.

An **xpub** (extended public key) is the same structure with the private
key stripped out — "neutered" — leaving only the public key and chain code.
This is exactly what makes xpubs useful for **watch-only** wallets: software
holding only an xpub can derive every one of its normal (non-hardened)
child public keys and addresses, letting it monitor balances and generate
new receiving addresses, without ever holding anything capable of signing
a transaction or moving funds.

**Normal (non-hardened) derivation** is possible from an xpub alone because
BIP32's normal child derivation formula only combines the parent's *public*
key, its chain code, and the child index — no private key is required
anywhere in that computation. **Hardened derivation**, by contrast, mixes in
the parent's actual *private* key as part of the derivation input — this is
a deliberate design choice specifically to prevent a known BIP32 weakness:
if an attacker had a parent xpub and just *one* of its normal children's
private keys, they could mathematically recover the parent's private key
and therefore every other child. Hardened derivation breaks that
relationship entirely, but the trade-off is that it can only ever be
computed from the parent's xpriv — never from its xpub — which is exactly
why `derive_normal_child_xpub` in this lab only accepts a non-hardened
index, and why `path_contains_hardened_step` matters: knowing whether a
path is hardened tells you whether an xpub-only wallet can even reach that
branch at all.

