# Lab 09 — BIP44 path decoding

## Commands used

```bash
cargo test --test lab_09 -- --nocapture
```

## Terminal output

```
running 4 tests
test changes_only_the_final_index ... ok
test explains_zero_based_account_and_chain ... ok
test decodes_every_bip44_level ... ok
test derives_the_selected_bip44_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Evidence references

`src/labs/lab09_bip44.rs`:

- `decode_bip44_path("m/44'/0'/2'/1/5")` parses the `DerivationPath` and requires
  exactly five steps — the first three (`purpose`, `coin_type`, `account`) hardened,
  the last two (`change`, `index`) normal — returning
  `Bip44PathInfo { purpose: 44, coin_type: 0, account: 2, change: 1, index: 5 }`.
- `describe_bip44_path` turns that struct into a sentence naming the third account
  (account index 2, zero-based), the change chain (`change == 1`), and the sixth
  address (index 5, zero-based).
- `with_address_index` rebuilds the path string with only the final component
  replaced, preserving every hardened apostrophe:
  `"m/44'/0'/2'/1/5"` + new index `6` → `"m/44'/0'/2'/1/6"`.
- `derive_bip44_address` derives the child extended key at the given path from the
  public test mnemonic and returns its P2PKH address; on `Regtest` it's asserted to
  start with `m` or `n`, and to be identical across two separate derivations from the
  same inputs.

## Explanation

`m / purpose' / coin_type' / account' / change / address_index` — everything below
`account'` is zero-based. `2'` is the third account, not the second. `address_index =
5` is the sixth address on that branch. Easy to get wrong once and then keep getting
wrong.

The apostrophes on `purpose'`, `coin_type'`, `account'` mark hardened derivation, and
that's not arbitrary — those levels define the tree's account boundaries, and
hardening them means a leaked private key at that level (or an xpriv) doesn't expose
the parent, the way a leaked normal child key plus parent xpub would (see Lab 08).
`change` and `address_index` stay non-hardened on purpose, because that's exactly
what lets an account-level xpub derive every receive and change address for a
watch-only wallet without any private key material involved.

`change` itself is just the branch switch: `0` is external (addresses handed out to
receive payments), `1` is internal (the wallet's own change, going back to itself).
Splitting those onto separate branches is what lets a watch-only wallet or explorer
tell "money coming in" from "change coming back" purely from which branch an address
was derived on — no other signal needed.
