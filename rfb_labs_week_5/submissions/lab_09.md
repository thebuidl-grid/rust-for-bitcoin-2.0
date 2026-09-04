# Lab 09 — BIP44 path decoding and address derivation

## Commands used

```
cargo test --test lab_09 -- --nocapture
cargo fmt --check
```

Implementation: `src/labs/lab09_bip44.rs` — `decode_bip44_path`,
`describe_bip44_path`, `with_address_index`, `derive_bip44_address`.

## Terminal output

```
running 4 tests
test changes_only_the_final_index ... ok
test decodes_every_bip44_level ... ok
test explains_zero_based_account_and_chain ... ok
test derives_the_selected_bip44_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Observed behaviour:
- `decode_bip44_path("m/44'/0'/2'/1/5")` = `{ purpose: 44, coin_type: 0, account: 2,
  change: 1, index: 5 }`; paths without exactly 5 levels, or with the wrong
  hardened/normal pattern, are rejected.
- `describe_bip44_path` of that info contains "third account", "change", and "sixth address".
- `with_address_index("m/44'/0'/2'/1/5", 6)` = `"m/44'/0'/2'/1/6"` (branch preserved).
- `derive_bip44_address(test_mnemonic, "", "m/44'/1'/0'/0/0", Regtest)` returns a
  deterministic `m`/`n`-prefixed P2PKH address.

## Evidence references

- `src/labs/lab09_bip44.rs` — validates the `purpose'/coin'/account'/change/index` shape.
- `tests/lab_09.rs` — the five-field decode and the "changes only the final index" case.
- Terminal block above from `cargo test --test lab_09`.

## Explanation

BIP44 fixes a five-level meaning on top of BIP32: `m / purpose' / coin_type' /
account' / change / address_index`. **purpose'** is always `44'` and marks the tree
as BIP44 (49'/84' are the SegWit siblings). **coin_type'** namespaces the chain —
`0'` mainnet BTC, `1'` all testnets — so one seed never mixes coins. **account'**
is a zero-based user-facing wallet ("account #0" = index 0); hardening it means its
xpub can be exported without exposing siblings. **change** is the chain selector:
`0` = external addresses you hand out, `1` = internal change outputs, kept as a
normal level so the account xpub can derive both. **address_index** is the
zero-based leaf, also normal, walked upward with a gap limit during recovery.
Changing only the last number moves along the same branch and yields a fresh
address; changing any hardened level above it is effectively a different wallet.
