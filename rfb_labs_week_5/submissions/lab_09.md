# Lab 09 — BIP44 path decoding

## Commands used

- `cargo fmt`
- `cargo check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --test lab_09`

## Terminal output

The Lab 09 test suite completed successfully:

- `decodes_every_bip44_level ... ok`
- `changes_only_the_final_index ... ok`
- `explains_zero_based_account_and_chain ... ok`
- `derives_the_selected_bip44_address ... ok`

Test result: **4 passed; 0 failed.**

The test path `m/44'/0'/2'/1/5` was decoded into:

- Purpose: `44`
- Coin type: `0`
- Account: `2`
- Change: `1`
- Address index: `5`

Changing the address index from `5` to `6` produced:

`m/44'/0'/2'/1/6`

The disposable BIP39 test mnemonic was also used to deterministically derive a Regtest P2PKH address.

## Evidence references

- Lab 09 test output showing all 4 tests passing.
- `src/labs/lab09_bip44.rs` contains the path decoding, description, index replacement, and address derivation implementations.

## Explanation

BIP44 gives wallets a standard way to organize the keys and addresses derived from a BIP32 master key. The path has five levels:

`m / purpose' / coin_type' / account' / change / address_index`

`m` represents the BIP32 master/root.

The first level, `44'`, identifies the BIP44 derivation scheme. The apostrophe means this level is hardened.

The second level is the coin type. Bitcoin mainnet uses `0`, while Bitcoin's test-network coin type is `1`. In this lab, the path uses `1` while the `Network::Regtest` argument controls how the final address is encoded.

The account level is zero-based. Therefore account `0` is the first account, account `1` is the second, and account `2` is the third account.

The change level selects the branch of the wallet. `0` is the external/receive branch, where addresses are normally given to other people to receive funds. `1` is the internal/change branch, used for change outputs returned to the wallet.

The final level is the address index, which is also zero-based. Index `0` is the first address, index `1` is the second, and index `5` is the sixth address.

For example:

`m/44'/0'/2'/1/5`

means the BIP44 scheme, Bitcoin coin type, third account, change branch, and sixth address.

The `with_address_index` function changes only the final address index while preserving the account and branch. This is important because changing another part of the path would select a different account or branch rather than simply generating the next address.

The final function takes the BIP39 mnemonic, derives the BIP32 master key, follows the requested BIP44 path, obtains the child private key and corresponding public key, and encodes that public key as a P2PKH address for the requested Bitcoin network.

The overall wallet derivation flow is therefore:

BIP39 mnemonic → seed → BIP32 master key → BIP44 derivation path → child key → public key → P2PKH address.

This connects the previous labs: Lab 07 introduced the mnemonic and seed, Lab 08 introduced BIP32 extended keys and derivation paths, and Lab 09 shows how a standardized BIP44 path selects a specific wallet address.