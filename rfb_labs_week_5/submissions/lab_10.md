# Lab 10 — Deterministic recovery across address families

**Author:** [Christopher Dominic Eze](https://github.com/Christopherdominic)

## Commands used

```bash
cargo test --test lab_10
cargo run --example evidence   # scratch script, deleted after copying the output below
```

## Terminal output

```
running 4 tests
test changing_only_the_index_changes_the_address ... ok
test identical_recovery_inputs_repeat ... ok
test format_selection_changes_the_lock_target ... ok
test derives_three_regtest_address_families ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Account 0, index 0, regtest, same class mnemonic and an empty passphrase:

```
DerivedAddressSet { bip44_p2pkh: "mkpZhYtJu2r87Js3pDiWJDmPte2NRZ8bJV", bip49_p2sh_p2wpkh: "2Mww8dCYPUpKHofjgcXcBCEGmniw9CoaiD2", bip84_p2wpkh: "bcrt1q6rz28mcfaxtmd6v789l9rrlrusdprr9pz3cppk" }
repeatable: true
index 0 vs 1 differ: true
```

## Evidence references

- `src/labs/lab10_recovery.rs` — `derive_address_for_path`, `derive_address_set`,
  `recovery_is_repeatable`, `changing_index_changes_address`.
- `tests/lab_10.rs::derives_three_regtest_address_families` — checks the three
  addresses start with `m`/`n`, `2`, and `bcrt1q` respectively, matching the
  legacy/P2SH-wrapped/native regtest prefixes shown above.
- `tests/lab_10.rs::identical_recovery_inputs_repeat` — same mnemonic, passphrase
  (`"class"`), path (`m/84'/1'/0'/0/0`), and network derived twice, addresses equal
  — that's the `repeatable: true` line.
- `tests/lab_10.rs::changing_only_the_index_changes_the_address` — index 0 vs. index
  1 on the same otherwise-identical path produce different addresses — the
  `index 0 vs 1 differ: true` line.

## Explanation

Recovery being deterministic comes straight out of how every step in the derivation
chain works: `Mnemonic::to_seed` is a pure function of the mnemonic words and the
passphrase (PBKDF2-HMAC-SHA512, no randomness), `Xpriv::new_master` is a pure
function of the seed (and which network's version bytes to use), and each BIP32
derivation step is a pure function of the parent key, its chain code, and the child
index. There's no coin-flipping anywhere in that chain. So handing the exact same
mnemonic, passphrase, path, and network into the derivation twice has to produce the
exact same private key, and therefore the exact same address, both times — which is
exactly what `recovery_is_repeatable` demonstrates, and it's the entire reason BIP32
trees are useful for backup and recovery in the first place: write down 12 words
(plus remember the passphrase, if any), and the whole tree of keys can be
reconstructed from scratch on any compatible wallet.

But "same mnemonic reproduces the same wallet" is only half the story, and this lab's
`derive_address_set` makes the other half concrete: the same mnemonic and passphrase
feed three completely different, non-overlapping trees of addresses depending on
which path and script convention you derive with. `m/44'/.../0/0` with P2PKH gives
you one address, `m/49'/.../0/0` wrapped in P2SH-P2WPKH gives you a different one, and
`m/84'/.../0/0` as native P2WPKH gives you a third — all from the same root key
material, all valid, and none of them the same coins as each other from the network's
point of view. A wallet doesn't just need the seed to restore someone's funds
correctly; it also needs to know (or guess, or ask) which purpose/coin-type/script
convention that seed was originally used with, or it'll faithfully reconstruct a
tree of addresses nothing was ever sent to. That's why wallet software cares so much
about which "wallet type" you pick when restoring from a seed phrase — the mnemonic
alone under-specifies the wallet.
