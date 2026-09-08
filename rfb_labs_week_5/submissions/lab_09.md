# Lab 09 — BIP44 path decoding

## Commands used

```bash
cargo test --test lab_09 -- --nocapture
cargo fmt --check
cargo clippy --all-targets
```

Implementation lives in `src/labs/lab09_bip44.rs`: `decode_bip44_path`,
`describe_bip44_path`, `with_address_index`, and `derive_bip44_address`.

## Terminal output

```
running 4 tests
test changes_only_the_final_index ... ok
test decodes_every_bip44_level ... ok
test explains_zero_based_account_and_chain ... ok
test derives_the_selected_bip44_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

`decodes_every_bip44_level` decodes `m/44'/0'/2'/1/5` into
`purpose=44, coin_type=0, account=2, change=1, index=5`;
`explains_zero_based_account_and_chain` confirms the generated description
names account 2 the "third account", change 1 the change branch, and index 5
the "sixth address".

## Evidence references

- `src/labs/lab09_bip44.rs` — `decode_bip44_path` enforces that purpose, coin
  type, and account are hardened while change and index are not (rejecting a
  malformed path otherwise), and `with_address_index` replaces only the final
  child.
- `tests/lab_09.rs` — `derives_the_selected_bip44_address` derives
  `m/44'/1'/0'/0/0` from the class mnemonic on Regtest twice and checks both
  results match, matching legacy `m`/`n` addresses.
- `bash grader/grade.sh` recorded `09 | 4/4 | 4 | ...` for this lab.

## Explanation

`m/44'/0'/2'/1/5` breaks into five fixed levels: `44'` is the purpose,
fixing this as a BIP44 path; `0'` is the coin type (0 = Bitcoin mainnet in the
SLIP-44 registry, though the lab derives on Regtest, which reuses testnet-style
addresses); `2'` is the account — the third account, because BIP44 accounts,
like everything else in this path, are zero-based, so account `0` is the
first account a wallet ever creates and account `2` is the third one a user
opened; `1` is the change flag (`0` = external/receive chain the user hands
out to be paid, `1` = internal/change chain the wallet uses privately for its
own change outputs); and `5` is the address index within that chain — again
zero-based, so index `5` is the sixth address ever derived on that branch, not
the fifth. `with_address_index` only ever rewrites this last, non-hardened
level, leaving purpose/coin/account/change untouched, which is exactly how a
wallet walks forward through a receive or change chain issuing new addresses.

The apostrophe (`'`, sometimes written `h`) marks *hardened* derivation for a
level, meaning child index ≥ 2³¹ was used and the parent's private key (not
just its public key) was required to compute it. BIP44 hardens purpose, coin
type, and account specifically so a leaked account-level `xpub` can never be
combined with any leaked child private key to work backward to the wallet's
master private key — `decode_bip44_path` enforces exactly this shape,
rejecting any path where those first three levels aren't hardened or where
change/index are hardened when they shouldn't be, since non-hardened
derivation is what makes the receive/change `xpub`s usable for watch-only
address generation in the first place.
