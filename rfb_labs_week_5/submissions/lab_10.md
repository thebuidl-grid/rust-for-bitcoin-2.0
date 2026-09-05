# Lab 10 — Deterministic recovery across address families

## Commands used

- `cargo fmt`
- `cargo check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test lab_10`
- `cargo test --test lab_10 print_regtest_address_set -- --nocapture`

## Terminal output

The Lab 10 test suite completed successfully:

- `changing_only_the_index_changes_the_address ... ok`
- `identical_recovery_inputs_repeat ... ok`
- `format_selection_changes_the_lock_target ... ok`
- `derives_three_regtest_address_families ... ok`

Test result: **4 passed; 0 failed.**

Using the public BIP39 test mnemonic with an empty passphrase, account `0`, address index `0`, and `Network::Regtest`, the three address families produced:

- BIP44 P2PKH: `mkpZhYtJu2r87Js3pDiWJDmPte2NRZ8bJV`
- BIP49 P2SH-P2WPKH: `2Mww8dCYPUpKHofjgcXcBCEGmniw9CoaiD2`
- BIP84 P2WPKH: `bcrt1q6rz28mcfaxtmd6v789l9rrlrusdprr9pz3cppk`

The repeatability test confirmed that using the same mnemonic, passphrase, derivation path, address format, and network produces the same address again.

The index-change test confirmed that changing only the final derivation index produces a different address.

## Evidence references

- Lab 10 test output showing all 4 tests passing.
- Terminal output showing the three deterministically derived Regtest address families.
- `src/labs/lab10_recovery.rs` contains the recovery and address derivation implementation.

## Explanation

Lab 10 brings together the wallet derivation concepts from Labs 07, 08, and 09.

A recovery phrase by itself is not an address. The BIP39 mnemonic is converted into a deterministic seed. BIP32 then uses that seed to create the master extended private key and derive child keys along a specified path.

The derivation path matters because different paths select different branches of the key tree. BIP44, BIP49, and BIP84 use different purpose values:

- BIP44 uses `44'` and produces legacy P2PKH addresses.
- BIP49 uses `49'` and produces P2SH-wrapped SegWit P2WPKH addresses.
- BIP84 uses `84'` and produces native SegWit P2WPKH addresses.

For this lab, the test-network coin type is `1`, account `0` is used, and the receive branch is `0`. The final address index is `0`.

Therefore the three paths are:

`m/44'/1'/0'/0/0`

`m/49'/1'/0'/0/0`

`m/84'/1'/0'/0/0`

Although these paths start from the same recovery root, the different purpose levels cause different child keys to be selected. The address format also determines how the resulting public key is turned into an address.

The resulting Regtest addresses demonstrate the three formats:

- P2PKH begins with `m` or `n`.
- P2SH-P2WPKH begins with `2`.
- Native P2WPKH uses the `bcrt1q` Bech32 format.

Deterministic recovery means that the same inputs always reproduce the same result. The recovery inputs include the mnemonic, passphrase, derivation path, address format, and network. If any of these relevant inputs changes, the resulting wallet/address selection can change.

The passphrase is particularly important because, as demonstrated in Lab 07, changing the BIP39 passphrase changes the seed. This means the same mnemonic with a different passphrase can represent a completely different wallet.

The final test also demonstrates address indexing. Changing only the final index from `0` to `1` selects another child key, and therefore another address, while keeping the same wallet account and receive branch.

The complete recovery flow covered by these labs is:

BIP39 mnemonic + passphrase → seed → BIP32 master key → derivation path → child key → public key → selected address format → Bitcoin address.

This is why deterministic wallets can recover the same addresses on another wallet implementation without storing every individual address: the addresses can be regenerated from the same recovery inputs and derivation conventions.