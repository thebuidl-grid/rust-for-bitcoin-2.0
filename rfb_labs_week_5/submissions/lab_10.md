# Lab 10 — Deterministic recovery across address families

## Commands used

```
cargo test --test lab_10 -- --nocapture
```

Ad-hoc check deriving account 0, index 0 on regtest from the public test mnemonic
across all three script families, and re-checking repeatability and index sensitivity:

```rust
lab10_recovery::derive_address_set(MNEMONIC, "", 0, 0, Network::Regtest)
lab10_recovery::recovery_is_repeatable(MNEMONIC, "class", "m/84'/1'/0'/0/0", AddressFormat::P2wpkh, Network::Regtest)
lab10_recovery::changing_index_changes_address(MNEMONIC, "", "m/84'/1'/0'/0/0", "m/84'/1'/0'/0/1", AddressFormat::P2wpkh, Network::Regtest)
```

## Terminal output

```
running 4 tests
test changing_only_the_index_changes_the_address ... ok
test identical_recovery_inputs_repeat ... ok
test format_selection_changes_the_lock_target ... ok
test derives_three_regtest_address_families ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
```

```
DerivedAddressSet {
    bip44_p2pkh: "mkpZhYtJu2r87Js3pDiWJDmPte2NRZ8bJV",
    bip49_p2sh_p2wpkh: "2Mww8dCYPUpKHofjgcXcBCEGmniw9CoaiD2",
    bip84_p2wpkh: "bcrt1q6rz28mcfaxtmd6v789l9rrlrusdprr9pz3cppk",
}
```

`recovery_is_repeatable` returns `true` for identical mnemonic/passphrase/path/format/
network, and `changing_index_changes_address` returns `true` when only the final
index differs between two otherwise identical paths.

## Evidence references

- `cargo test --test lab_10` output above.
- Source: `src/labs/lab10_recovery.rs`.
- Test suite: `tests/lab_10.rs`.
- All addresses are derived from the public test mnemonic on regtest; none are
  production wallet addresses.

## Explanation

BIP32 derivation is a pure function: the same seed (mnemonic plus passphrase) and the
same derivation path always produce the same extended key, because each step is
HMAC-SHA512 over the parent's key material and chain code with no external
randomness involved. That is why `recovery_is_repeatable` always returns `true` for
identical inputs, and it is the entire basis for HD wallet recovery — a wallet can be
rebuilt from twelve words alone. But the mnemonic and passphrase only reproduce the
private key tree; restoring the *wallet a person actually used* also depends on
agreeing which paths and script types that wallet's software chose. BIP44, BIP49, and
BIP84 fix the purpose field (44'/49'/84') and the resulting script type (P2PKH,
P2SH-wrapped P2WPKH, native P2WPKH) for exactly this reason: two wallets could derive
identical private keys from the same seed at `m/0/0/0/0/0` and still show completely
different addresses if one encodes it as P2PKH and the other as P2WPKH, as
`format_selection_changes_the_lock_target` demonstrates. Recovering funds correctly
therefore means reproducing the seed, the coin type, the account and change branch,
the address index, and the script-type convention the original wallet used, not just
the seed.
