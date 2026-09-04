# Lab 10 — Deterministic recovery across BIP44, BIP49, and BIP84

## Commands used

```
cargo test --test lab_10 -- --nocapture
cargo fmt --check
```

Implementation: `src/labs/lab10_recovery.rs` — `derive_address_for_path`,
`derive_address_set`, `recovery_is_repeatable`, `changing_index_changes_address`.

## Terminal output

```
running 4 tests
test changing_only_the_index_changes_the_address ... ok
test identical_recovery_inputs_repeat ... ok
test format_selection_changes_the_lock_target ... ok
test derives_three_regtest_address_families ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Observed behaviour with the public test mnemonic on `Network::Regtest`:
- `derive_address_set(m, "", account = 0, index = 0)` returns a BIP44 address
  (`m`/`n...`), a BIP49 address (`2...`), and a BIP84 address (`bcrt1q...`), from the
  paths `m/44'/1'/0'/0/0`, `m/49'/1'/0'/0/0`, `m/84'/1'/0'/0/0`.
- `recovery_is_repeatable(m, "class", "m/84'/1'/0'/0/0", P2wpkh, Regtest)` = true.
- `changing_index_changes_address(.., ".../0/0", ".../0/1", ..)` = true.
- Same path, different `AddressFormat` (P2pkh vs P2wpkh) => different address strings.

## Evidence references

- `src/labs/lab10_recovery.rs` — one derivation helper, format chosen by `AddressFormat`
  (`p2pkh`, `p2shwpkh`, `p2wpkh`, `p2tr`), coin type `1` off mainnet.
- `src/labs/common.rs` — the shared mnemonic -> seed -> master -> child pipeline.
- `tests/lab_10.rs` — the three-family set, repeatability, and index-sensitivity checks.

## Explanation

Recovery is deterministic because every step is a pure function. The **inputs** are
the mnemonic, the passphrase, and the derivation path; BIP39 turns the first two
into a fixed 512-bit seed, BIP32 turns the seed into a fixed master xpriv, and each
path step is a fixed HMAC-SHA512. Nothing is random at restore time, so identical
inputs always reproduce the identical key — and therefore the identical address —
which is what `recovery_is_repeatable` shows. The **derivation conventions** are
what make a wallet portable between implementations: BIP44 (`44'`, P2PKH), BIP49
(`49'`, P2SH-wrapped P2WPKH), and BIP84 (`84'`, native P2WPKH) all branch from the
same seed but agree on which path maps to which script type and address encoding.
So one 12-word backup deterministically regenerates three parallel address
families; the script family only changes how the final public key is wrapped into
an address, not the key itself, which is why the same path under two formats yields
two different strings.
