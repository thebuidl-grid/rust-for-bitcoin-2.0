# Lab 09 — BIP44 path decoding

## Commands used

I formatted the implementation and ran the focused and complete BIP44 tests:

```bash
cargo fmt
cargo test --test lab_09 decodes_every_bip44_level
cargo test --test lab_09 explains_zero_based_account_and_chain
cargo test --test lab_09 changes_only_the_final_index
cargo test --test lab_09 derives_the_selected_bip44_address
cargo test --test lab_09
```

## Terminal output

The path decoder and address derivation produced these disposable results:

```text
decoded m/44'/0'/2'/1/5:
  purpose: 44
  coin_type: 0
  account: 2
  change: 1
  index: 5

index change:
  m/44'/0'/2'/1/5 -> m/44'/0'/2'/1/6

regtest address at m/44'/1'/0'/0/0:
  mkpZhYtJu2r87Js3pDiWJDmPte2NRZ8bJV

test result: ok. 4 passed; 0 failed
```

## Evidence references

- Implementation: `src/labs/lab09_bip44.rs`
- Public tests: `tests/lab_09.rs`
- The decoder enforces five levels, hardened purpose/coin/account components,
  normal receive/change and index components, purpose `44'`, and a branch value of
  either 0 or 1.
- Address derivation uses only the published mnemonic on regtest.

## Explanation

A BIP44 path has the structure
`m / purpose' / coin_type' / account' / change / address_index`. Purpose `44'`
identifies the BIP44 convention. Coin type separates networks or assets; Bitcoin
mainnet uses 0, while Bitcoin test networks conventionally use 1. Account is a
zero-based hardened subtree, so account 2 is the third account.

The change level is normal: 0 selects the external receiving branch and 1 selects
the internal change branch. The final normal index selects one key and address on
that branch, so index 5 is the sixth address. Purpose, coin type, and account are
hardened to isolate higher-level subtrees, while change and address index remain
normal so an account xpub can generate watch-only receiving and change addresses.
