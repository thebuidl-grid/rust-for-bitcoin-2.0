# Lab 08 — BIP32 extended keys and hardened derivation

## Commands used

```bash
cargo test --test lab_08 -- --nocapture
```

## Terminal output

```
running 4 tests
test distinguishes_hardened_and_normal_paths ... ok
test derives_matching_extended_keys ... ok
test xpub_derives_a_normal_public_child ... ok
test creates_a_test_family_master_xpriv ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Evidence references

`src/labs/lab08_bip32.rs`, always run against the public test mnemonic on `Regtest`:

- `master_xpriv` derives the BIP39 seed then `Xpriv::new_master(network, &seed)`; the
  test confirms it's deterministic (same inputs → same `tprv...` string every time) and
  that regtest/testnet keys use the `tprv` prefix.
- `derive_extended_keys` parses a `DerivationPath` (e.g. `m/84'/1'/0'`), calls
  `master.derive_priv(&secp, &path)`, then `Xpub::from_priv(&secp, &xpriv)` to "neuter"
  it into the public-only extended key (`tpub...`).
- `derive_normal_child_xpub` takes only an `Xpub` string and a non-hardened index, and
  calls `parent.derive_pub(&secp, &[ChildNumber::from_normal_idx(index)?])` — no
  private key material is ever read or required.
- `path_contains_hardened_step` parses the path and checks `ChildNumber::is_hardened()`
  on every step; `m/44'/0'/0'/0/0` reports `true` (its first three steps are hardened),
  `m/0/1/2` reports `false`, and a malformed path like `"not/a/path"` returns `Err(..)`.

`Xpub::derive_pub` succeeding directly from the parent xpub — with no seed, mnemonic,
or xpriv involved — is itself the evidence that normal children are derivable from
public data alone; the hardened case is not exercised here precisely because BIP32
provides no public-only path to it, which the explanation below covers.

## Explanation

Chain code: 256 extra bits carried alongside every extended key, mixed into
HMAC-SHA512 at each derivation step along with the parent key. Without it, deriving a
"child" would just be an index over the parent's public key — and EC point math is
public and reversible, so anyone could reproduce the same children from the parent
pubkey and a guessed index. The chain code is what makes derivation depend on
something not recoverable from the public key alone.

xpub is public key + chain code, and that's exactly what watch-only use needs: an
exchange or a block explorer holding only the xpub can derive every non-hardened
child pubkey and every address the account will ever produce, so it can watch for
incoming payments and hand out fresh receive addresses — but it can't sign anything,
because it never has the private key.

Hardened children can't come from an xpub because BIP32 defines hardened derivation
(CKDpub for hardened indices doesn't even exist) to hash the parent's *private* key
into HMAC-SHA512, not the public key. That's deliberate — deriving from the pubkey
would mean that if one child's private key and the parent xpub ever both leaked, the
parent's chain code (and for normal derivation, the parent private key too) becomes
recoverable, and the whole sibling tree goes with it. Hardened derivation breaks that
chain: leaking one hardened child tells you nothing about its siblings or its parent.
Which is why BIP44 hardens everything at account level and above.
