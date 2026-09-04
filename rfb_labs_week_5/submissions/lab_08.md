# Lab 08 — BIP32 extended keys

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

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
```

## Evidence references

All four public tests in `tests/lab_08.rs` pass against `src/labs/lab08_bip32.rs`, derived from the
public test mnemonic on regtest only: `master_xpriv` produces a deterministic `tprv...` key from
the seed; `derive_extended_keys("m/84'/1'/0'")` produces a matching `tprv.../tpub...` pair at that
path; `derive_normal_child_xpub` derives child index 7 from a parent `xpub` and produces a
*different* `tpub...` with no private key material involved; `path_contains_hardened_step`
correctly flags `m/44'/0'/0'/0/0` as hardened, `m/0/1/2` as not, and rejects a malformed path.

## Explanation

**Chain code:** every extended key (xpriv/xpub) carries, alongside its actual key, a 32-byte
*chain code* that is mixed into the HMAC-SHA512 computation for every child derivation. Its
purpose is entropy separation: without it, deriving children would just be some simple
deterministic function of the parent key alone, and — critically — knowing one child private key
plus the parent's *public* key would let an attacker work backwards to the parent's chain code and
compromise every sibling key. The chain code makes each derivation step depend on 256 extra bits
that are never derivable from the key material alone, so compromising one child key doesn't
compromise its siblings or the rest of the tree.

**Watch-only xpubs:** `derive_normal_child_xpub` shows the payoff — a raw `xpub`/`tpub` and its
chain code are enough to derive every *normal* (non-hardened) child's public key with
`Xpub::ckd_pub`, and nothing else. That means a server, watch-only wallet, or auditor can generate
a whole tree of receiving addresses to monitor for incoming payments, without ever holding a
private key that could sign anything — hence "watch-only."

**Why hardened children can't come from an xpub:** normal derivation computes the child using
`HMAC-SHA512(chain_code, serialized_parent_PUBLIC_key || index)` — it only ever needs the parent's
public key, which is why `ckd_pub` works from an xpub alone. Hardened derivation instead uses
`HMAC-SHA512(chain_code, 0x00 || parent_PRIVATE_key || index)` — it needs the parent's *private*
key as direct HMAC input, specifically so that this derivation step *cannot* be performed from
public data alone. `path_contains_hardened_step` distinguishes exactly this: a hardened index
(`44'`) requires the private key to derive, while a normal index (`0`, `1`, `2`, ...) does not.
This is also precisely why account-level keys (and above) are conventionally hardened in BIP44 —
it stops the "leak one child xprv + the parent xpub → recover the whole subtree" attack that would
otherwise apply to non-hardened derivation.
