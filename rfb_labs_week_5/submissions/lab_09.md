# Lab 09 — BIP44 path decoding

## Commands used

```bash
cargo test --test lab_09
cargo fmt --check
```

## Terminal output

```text
$ cargo test --test lab_09
running 4 tests
test decodes_every_bip44_level ... ok
test changes_only_the_final_index ... ok
test explains_zero_based_account_and_chain ... ok
test derives_the_selected_bip44_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

`cargo fmt --check` produced no diff.

## Evidence references

- Implementation: `src/labs/lab09_bip44.rs`
- Test suite: `tests/lab_09.rs`
- `decode_bip44_path("m/44'/0'/2'/1/5")` returns
  `Bip44PathInfo { purpose: 44, coin_type: 0, account: 2, change: 1, index: 5 }`.
- `describe_bip44_path` on that info produces a sentence containing "third account" (account 2 is
  the third, zero-based), the word "change", and "sixth address" (index 5 is the sixth).
- `with_address_index("m/44'/0'/2'/1/5", 6)` returns `"m/44'/0'/2'/1/6"`, changing only the final
  segment.
- `derive_bip44_address(MNEMONIC, "", "m/44'/1'/0'/0/0", Regtest)` returns an `m...`/`n...`
  regtest P2PKH address, deterministically, on every call — this is disposable test-family output
  from the public test mnemonic only.

## Explanation

`m/44'/0'/2'/1/5` reads left to right as increasingly specific scope. **Purpose** (`44'`) is a
constant that says "interpret everything after this as a BIP44 multi-account tree" — it lets a
wallet distinguish this convention from BIP49 (`49'`) or BIP84 (`84'`) trees built from the same
seed. **Coin type** (`0'`) selects which registered cryptocurrency this subtree belongs to (`0'`
is Bitcoin mainnet; test networks share `1'`), so one seed can safely hold independent trees for
multiple coins without key reuse across them. **Account** (`2'`) is a zero-based user-facing
ledger — "account 2" is a wallet's *third* account, the same way array index 2 is a list's third
element — letting one seed maintain several separate balances/histories (e.g. personal vs.
business) that don't share addresses. All three of these levels are **hardened** (the apostrophe),
which matters because hardened derivation requires the parent's private key, not just its xpub —
so even if an account-level xpub leaks, an attacker cannot climb back up to a sibling account or
to the coin/purpose level. **Change** (`1`) is the receive/change branch: `0` is the external chain
handed out to third parties as receiving addresses, `1` is the internal change chain a wallet uses
for its own change outputs, keeping the two visually and semantically separate even though both
are normal (non-hardened) derivation. **Index** (`5`) is the zero-based address slot within that
branch — the sixth address ever generated on that specific chain — and it's the only field
`with_address_index` needs to touch to move to a new receiving address without disturbing the
account or branch it belongs to.

