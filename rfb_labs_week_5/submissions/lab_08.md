# Lab 08 — BIP32 extended keys

## Commands used

```
cargo test --test lab_08 -- --nocapture
```

## Terminal output

```
running 4 tests
test distinguishes_hardened_and_normal_paths ... ok
test derives_matching_extended_keys ... ok
test xpub_derives_a_normal_public_child ... ok
test creates_a_test_family_master_xpriv ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s
```

## Evidence references

Using the public class test mnemonic with an empty passphrase on regtest (test/regtest
key material only — prefixes shown, full strings redacted per lab instructions):

```
master_xpriv                         starts with "tprv8Zgx..." (network = Test)
derive_extended_keys("m/84'/1'/0'")  xpriv starts with "tprv8fSj...", xpub starts with "tpubDC8m..."
```

Calling `master_xpriv` twice with identical inputs returned identical output, confirming
deterministic derivation. `derive_normal_child_xpub` on the resulting account-level xpub
at index 7 produced a different `tpub...` string than the parent, proving the xpub alone
can extend the public key tree one non-hardened level. `path_contains_hardened_step`
returned `true` for `m/44'/0'/0'/0/0` and `false` for `m/0/1/2`, and returned `Err` for
the malformed input `"not/a/path"`.

## Explanation

The chain code is 32 bytes of extra entropy carried alongside every extended key,
mixed into the HMAC-SHA512 input at each derivation step together with either the parent
public key (normal child) or parent private key (hardened child) and the child index.
Its purpose is to make each child key's derivation path-dependent and unpredictable from
the index alone — without it, sibling keys derived from the same parent public key would
be trivially related to each other and to the parent's public key via simple EC point
arithmetic. An xpub is "watch-only" because it carries the chain code and the public key
but no private key material: software holding only an xpub can derive every non-hardened
descendant's public key (and therefore every receiving address in that subtree) without
ever being able to produce a signature, which is exactly what an auditor or a
watch-only wallet needs. Hardened children cannot be derived from a parent xpub because
hardened derivation's HMAC input uses the parent *private* key (`0x00 || priv_key ||
index`) specifically to break the parent-pubkey-to-child-pubkey relationship — if a
normal-child leak of one child private key plus the parent xpub could reconstruct the
parent private key (which it can, for normal children, since the tweak is public), then
hardened derivation being immune to this requires never putting the parent public key
in the hash input for hardened steps at all, and by construction that same absence of
public-key material is what makes hardened children unreachable from an xpub — there is
simply no public information from which to reconstruct the same HMAC output.
