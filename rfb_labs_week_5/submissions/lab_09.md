# Lab 09 — BIP44 path decoding

## Commands used

```
cargo test --test lab_09 -- --nocapture
```

Ad-hoc check decoding, describing, and re-indexing `m/44'/0'/2'/1/5`, and deriving a
disposable address from the public test mnemonic:

```rust
lab09_bip44::decode_bip44_path("m/44'/0'/2'/1/5")
lab09_bip44::describe_bip44_path(&path_info)
lab09_bip44::with_address_index("m/44'/0'/2'/1/5", 6)
lab09_bip44::derive_bip44_address(MNEMONIC, "", "m/44'/1'/0'/0/0", Network::Regtest)
```

## Terminal output

```
running 4 tests
test changes_only_the_final_index ... ok
test decodes_every_bip44_level ... ok
test explains_zero_based_account_and_chain ... ok
test derives_the_selected_bip44_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

```
Bip44PathInfo { purpose: 44, coin_type: 0, account: 2, change: 1, index: 5 }
description = BIP44 path: purpose 44' selects BIP44, coin type 0' selects the chain, the third account (account index 2) holds the funds, the change chain (change 1) picks the branch, and the sixth address (address index 5) is the one in use.
with_address_index("m/44'/0'/2'/1/5", 6) = m/44'/0'/2'/1/6
bip44 address (m/44'/1'/0'/0/0, regtest, test mnemonic) = mkpZhYtJu2r87Js3pDiWJDmPte2NRZ8bJV
```

## Evidence references

- `cargo test --test lab_09` output above.
- Source: `src/labs/lab09_bip44.rs`.
- Test suite: `tests/lab_09.rs`.
- The derived address uses the public test mnemonic on regtest; it is a disposable
  test address, not a funded wallet.

## Explanation

BIP44 paths are `m / purpose' / coin_type' / account' / change / index`. `purpose'`
(44') marks the tree as following the BIP44 scheme; `coin_type'` (0' for Bitcoin
mainnet, 1' for any Bitcoin test network) picks the cryptocurrency; both are hardened
with an apostrophe so a leaked account-level xpub can never be used to climb back up
and derive a different coin's keys. `account'` is also hardened and is zero-based, so
`account = 2` is the *third* account a wallet has created, not the second. `change`
is the branch selector: `0` is the external/receiving chain handed out to other
people, `1` is the internal/change chain used only by the wallet itself for its own
change outputs; it is left non-hardened so an account-level xpub can still generate
watch-only addresses on both branches. `index` is the zero-based address counter
within that branch, so `index = 5` is the *sixth* address on that branch. Hardening
stops at the account level specifically to balance security (protecting the seed and
sibling accounts) against convenience (letting a single xpub per account serve as a
watch-only wallet for both its receive and change addresses).
