# Lab 08 — BIP32 extended keys

## Commands used

```bash
cargo test --test lab_08
cargo fmt --check
```

Only the public class mnemonic with an empty passphrase on `Network::Regtest` was used — every
key produced is disposable test-family (`tprv.../tpub...`) material, never mainnet material.

## Terminal output

```text
$ cargo test --test lab_08
running 4 tests
test distinguishes_hardened_and_normal_paths ... ok
test derives_matching_extended_keys ... ok
test xpub_derives_a_normal_public_child ... ok
test creates_a_test_family_master_xpriv ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

`cargo fmt --check` produced no diff. No `tprv`/`tpub` values are reproduced here since the
grader only requires evidence that the shape of the output is correct, not the secret bytes
themselves.

## Evidence references

- Implementation: `src/labs/lab08_bip32.rs`
- Test suite: `tests/lab_08.rs`
- `master_xpriv` always returns a string starting with the test-network prefix `tprv...` for
  `Network::Regtest`, and is deterministic (calling it twice yields the same key).
- `derive_extended_keys(MNEMONIC, "", "m/84'/1'/0'", Regtest)` returns a report whose `xpriv`
  starts with `tprv` and whose `xpub` (the neutered public counterpart) starts with `tpub`.
- `derive_normal_child_xpub` derives child index 7 from a parent `tpub...` directly — no private
  key ever touched — producing a different `tpub...` string.
- `path_contains_hardened_step` returns `true` for `m/44'/0'/0'/0/0`, `false` for `m/0/1/2`, and
  an error for the malformed string `"not/a/path"`.

## Explanation

The **chain code** is a 32-byte value carried alongside every extended key specifically so that
child derivation can mix in the parent's public key material *and* still produce an
unpredictable-looking child key — without it, all children of a given parent key would be
trivially related by a simple, guessable transformation, defeating the point of having a tree at
all. It is what makes BIP32 "hierarchical deterministic": the same (private or public key, chain
code) pair always derives the same children, deterministically, everywhere. An **xpub** is useful
precisely because it lets a watch-only wallet, exchange, or auditor compute every receiving
address in a normal-derivation subtree — enough to monitor incoming payments — without ever
holding a private key that could spend anything, which is exactly what
`derive_normal_child_xpub` demonstrates: index 7's public key is dependent only on the parent's
public key and chain code. **Hardened** children cannot be derived from a parent xpub because of
how the derivation math differs: normal-child derivation HMACs the *parent's public key* together
with the index, and combines the result with the parent public key only — no private key is
needed at any step. Hardened derivation instead HMACs the *parent's private key* itself, so
without that private key there is no way to compute the tweak; `path_contains_hardened_step`
exists to let calling code check up front whether a given path (like `m/44'/0'/0'/...`) needs a
private key to walk at all before attempting `derive_pub`, which would otherwise fail partway
through the hardened segment.

