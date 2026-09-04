# Lab 10 — Deterministic recovery across BIP44, BIP49, and BIP84

## Commands used

```bash
cargo test --test lab_10 -- --nocapture
```

## Terminal output

```
running 4 tests
test identical_recovery_inputs_repeat ... ok
test changing_only_the_index_changes_the_address ... ok
test format_selection_changes_the_lock_target ... ok
test derives_three_regtest_address_families ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Evidence references

`src/labs/lab10_recovery.rs`, all against the public test mnemonic on `Regtest` (coin
type `1'`):

- `derive_address_set(mnemonic, "", account, index, Regtest)` derives index 0 on all
  three receive branches from one recovery root:
  - `m/44'/1'/0'/0/0` → P2PKH, regtest prefix `m`/`n` (Base58Check, legacy script)
  - `m/49'/1'/0'/0/0` → P2SH-wrapped P2WPKH, regtest prefix `2` (Base58Check, P2SH
    outer script hiding a P2WPKH witness program)
  - `m/84'/1'/0'/0/0` → native P2WPKH, regtest prefix `bcrt1q` (Bech32, witness v0
    script)
- `recovery_is_repeatable` derives the same path twice from the same mnemonic,
  passphrase, format, and network and confirms the two addresses are byte-for-byte
  identical.
- `changing_index_changes_address` derives `.../0/0` and `.../0/1` from otherwise
  identical inputs and confirms the addresses differ.
- `derive_address_for_path` with the same path but `AddressFormat::P2pkh` vs
  `AddressFormat::P2wpkh` proves the script family, not just the key, controls the
  final address — same derived key, two different lock targets.

## Explanation

Every step from mnemonic to address is a pure function — no RNG, no clock, nothing
external anywhere in the chain. `to_seed` is deterministic PBKDF2 over fixed inputs.
`new_master` and every `derive_priv` step down a fixed path is deterministic
HMAC-SHA512. Turning a pubkey into an address is deterministic hashing and encoding.
Same mnemonic, same passphrase, same path — the output is mathematically forced to be
identical every time. That's the entire point of "deterministic" in HD wallets: 12 or
24 words standing in for an entire tree of keys, with nothing else to remember.

But the key being reproducible isn't the whole story of "recovery." The key at a
given path is always the same key — what address gets shown for it depends on the
script-family convention layered on top (BIP44 vs BIP49 vs BIP84), and that's a
software choice, not something baked into the derivation. `derive_address_for_path`
makes this concrete: the exact same derived key at `m/44'/1'/0'/0/0` produces a
different address depending on whether you encode it as P2PKH or P2WPKH. So if a
wallet restores a mnemonic assuming BIP84 but the coins were actually received at
BIP44 or BIP49 addresses from that same seed, it scans the wrong branch and reports a
zero balance — not because anything about the derivation failed, but because the
script-family convention it guessed doesn't match the one that was actually used.
