# Lab 08 — BIP32 extended keys

**Author:** [Christopher Dominic Eze](https://github.com/Christopherdominic)

## Commands used

```bash
cargo test --test lab_08
cargo run --example evidence   # scratch script, deleted after copying the output below
```

Only the published class mnemonic (`abandon x11 about`) and an empty passphrase were
used — this is disposable test material, but I still redacted the xprivs below to
practice not pasting full private material.

## Terminal output

```
running 4 tests
test distinguishes_hardened_and_normal_paths ... ok
test derives_matching_extended_keys ... ok
test xpub_derives_a_normal_public_child ... ok
test creates_a_test_family_master_xpriv ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

```
master xpriv (redacted): tprv8ZgxMBic...(111 chars total)
path:  m/84'/1'/0'
xpriv (redacted): tprv8fSjiqEQ...(111 chars total)
xpub:  tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M
child xpub (index 7): tpubDFd87GgwwqSReUu4pKLNFcrjb5SMPesTb6MjEL6FULEtbc4nqbZZqwomwZeGfpuZwr5GgyA9rPhuJq242jhuALhg6KgZcZMovz6o6JYK2FV
m/44'/0'/0'/0/0 has a hardened step: true
```

## Evidence references

- `src/labs/lab08_bip32.rs` — `master_xpriv`, `derive_extended_keys`,
  `derive_normal_child_xpub`, `path_contains_hardened_step`.
- `tests/lab_08.rs::creates_a_test_family_master_xpriv` — asserts the master xpriv
  starts with `tprv` (regtest/testnet version bytes) and is identical across two
  calls with the same mnemonic/passphrase/network, which the redacted line above
  confirms (both runs produced `tprv8ZgxMBic...`).
- `tests/lab_08.rs::xpub_derives_a_normal_public_child` — derives child index 7 from
  the `m/84'/1'/0'/0` xpub with no private key involved, and the resulting xpub
  differs from the parent — that's the `child xpub (index 7)` line above.
- `tests/lab_08.rs::distinguishes_hardened_and_normal_paths` — `m/44'/0'/0'/0/0`
  reports `true` for having a hardened step (the first three levels are hardened),
  matching the last line above.

## Explanation

The chain code is a 32-byte value carried alongside every extended key, and its whole
purpose is to make child key derivation deterministic without leaking information it
shouldn't. When you derive a child, BIP32 HMAC-SHA512s the chain code together with
either the parent's public key (normal derivation) or private key (hardened
derivation) and the child index, and splits the result into a new 32-byte tweak plus
a new chain code for the child. Without the chain code, "derive child #7" would either
be undefined or would have to be based on the key alone, which would make sibling
keys correlate with each other in ways that leak structure. The chain code is what
lets the same index produce a completely different, unrelated-looking child key under
a different parent.

Xpubs being usable for watch-only wallets falls straight out of how derivation works
for non-hardened children: `derive_normal_child_xpub` in this lab does exactly that —
it takes a public key, its chain code, and a child index, and produces a child public
key and chain code using only public-key elliptic-curve math (point addition with a
tweak derived from the parent public key and chain code). No private key ever enters
that computation. That means someone can hand out an xpub — say, to a piece of
accounting or monitoring software — and that software can generate every receiving
address the wallet will ever use, watch the chain for payments to them, and report
balances, all without ever being able to sign a transaction or move a single coin.

Hardened derivation breaks that property on purpose. A hardened child's tweak is
computed from `HMAC-SHA512(chain_code, 0x00 || parent_private_key || index)` — the
parent's *private* key is mixed directly into the hash input, not just its public
key. There's no way to reconstruct that HMAC input from the public key alone, because
the public key doesn't determine the private key (that's the entire point of elliptic
curve cryptography being one-way). So an xpub, on its own, simply doesn't contain
enough information to derive a hardened child — you would need the parent xprv. This
is also a deliberate security feature, not just an incidental limitation: if hardened
children *could* be derived from an xpub, then leaking a single non-hardened child
private key together with the parent xpub would let an attacker walk back up to the
parent's master private key (this is the well-known xpub + one leaked child privkey
attack). Hardening the account-level and above steps of BIP44/49/84 paths is exactly
what prevents that.
