# Lab 10 — Deterministic recovery across address families

## Commands used

I formatted the implementation, ran each recovery behavior test, and then ran the
complete Lab 10 suite:

```bash
cargo fmt
cargo test --test lab_10 derives_three_regtest_address_families
cargo test --test lab_10 identical_recovery_inputs_repeat
cargo test --test lab_10 changing_only_the_index_changes_the_address
cargo test --test lab_10 format_selection_changes_the_lock_target
cargo test --test lab_10
```

## Terminal output

The same public mnemonic, empty passphrase, account 0, index 0, and regtest network
produced:

```text
BIP44 P2PKH:
mkpZhYtJu2r87Js3pDiWJDmPte2NRZ8bJV

BIP49 P2SH-P2WPKH:
2Mww8dCYPUpKHofjgcXcBCEGmniw9CoaiD2

BIP84 P2WPKH:
bcrt1q6rz28mcfaxtmd6v789l9rrlrusdprr9pz3cppk

identical inputs reproduced the same address: true
changing only the final index produced a different address: true

test result: ok. 4 passed; 0 failed
```

## Evidence references

- Implementation: `src/labs/lab10_recovery.rs`
- Shared BIP32 derivation: `src/labs/lab08_bip32.rs`
- Public tests: `tests/lab_10.rs`
- All addresses use the published disposable BIP39 mnemonic and regtest encoding.
- Unsupported Taproot and unknown format selections return explicit errors rather
  than silently deriving the wrong address family.

## Explanation

Deterministic recovery requires the same mnemonic, optional passphrase, derivation
path, address format, and network convention. The mnemonic and passphrase reproduce
the BIP39 seed; BIP32 reproduces the same child key at a given path; and the selected
script family encodes that key as P2PKH, wrapped P2WPKH, or native P2WPKH. Identical
inputs therefore reproduce identical addresses without storing every child key
individually.

BIP44, BIP49, and BIP84 use different purpose branches for legacy P2PKH, wrapped
SegWit, and native SegWit. Changing only the final index selects a sibling key and a
different address. A wrong passphrase, path, account, index, script family, or
network can produce a valid but different wallet view. BIP39 words alone do not
record all of these discovery conventions, so reliable backups should also retain
the wallet's derivation and script-type information.
