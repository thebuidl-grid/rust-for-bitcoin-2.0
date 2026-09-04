# Lab 08 — BIP32 extended keys

## Commands used

```
cargo test --test lab_08 -- --nocapture
```

Ad-hoc check using only the public BIP39 test mnemonic and an empty passphrase (no
real recovery material):

```rust
lab08_bip32::master_xpriv(MNEMONIC, "", Network::Regtest)
lab08_bip32::derive_extended_keys(MNEMONIC, "", "m/84'/1'/0'", Network::Regtest)
lab08_bip32::derive_normal_child_xpub(&parent.xpub, 7)
lab08_bip32::path_contains_hardened_step("m/44'/0'/0'/0/0")
```

## Terminal output

```
running 4 tests
test distinguishes_hardened_and_normal_paths ... ok
test derives_matching_extended_keys ... ok
test xpub_derives_a_normal_public_child ... ok
test creates_a_test_family_master_xpriv ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

Redacted prefixes and derivation observations (test-network keys derived from the
public test mnemonic, not real recovery material):

```
master xpriv prefix    = tprv8ZgxMBic... (network kind = testnet, depth = 0)
m/84'/1'/0' xpriv prefix = tprv8fSjiqEQ... (starts with tprv, network = regtest/testnet)
m/84'/1'/0' xpub prefix  = tpubDC8msFGe... (starts with tpub)
```

`derive_normal_child_xpub` on the `m/84'/1'/0'/0` xpub at index 7 returns a different
`tpub...` string than the parent xpub. `path_contains_hardened_step("m/44'/0'/0'/0/0")`
returns `true`; `path_contains_hardened_step("m/0/1/2")` returns `false`;
`path_contains_hardened_step("not/a/path")` returns `Err`.

## Evidence references

- `cargo test --test lab_08` output above.
- Source: `src/labs/lab08_bip32.rs`.
- Test suite: `tests/lab_08.rs`.
- Only redacted prefixes of test-network keys derived from the public mnemonic are
  shown; no full xpriv/xpub was recorded.

## Explanation

The chain code is 32 bytes of extra entropy carried alongside every extended key,
mixed into the HMAC-SHA512 input at every derivation step so that child keys are not
simply "the parent key plus an index" but a keyed, deterministic function of the
parent's private (or public) key *and* its chain code together. Without it, anyone
seeing one child private key and the parent public key could potentially work
backward toward siblings; the chain code is what makes each derivation step
depend on secret-looking randomness even though the whole tree is fully
deterministic. An xpub's watch-only property comes from `Xpub::derive_pub`/`ckd_pub`
using only public-key EC point addition for *normal* children, so a wallet or a
block explorer holding just an xpub can enumerate every receiving address in that
branch and watch for incoming payments without ever holding a private key capable of
spending them. Hardened derivation breaks that: it folds the *parent's private key*
directly into the HMAC input instead of the parent's public key, specifically so
that leaking one hardened child's private key plus the parent xpub cannot be
combined to reconstruct the parent private key (a real risk with normal/non-hardened
derivation). Since computing a hardened child requires the parent's private key, an
xpub alone can never derive a hardened child, which is why every BIP44/49/84 path
hardens the purpose, coin type, and account levels.
