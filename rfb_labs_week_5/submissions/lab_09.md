# Lab 09 — BIP44 path decoding

**Author:** [Christopher Dominic Eze](https://github.com/Christopherdominic)

## Commands used

```bash
cargo test --test lab_09
cargo run --example evidence   # scratch script, deleted after copying the output below
```

## Terminal output

```
running 4 tests
test changes_only_the_final_index ... ok
test decodes_every_bip44_level ... ok
test explains_zero_based_account_and_chain ... ok
test derives_the_selected_bip44_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

```
Bip44PathInfo { purpose: 44, coin_type: 0, account: 2, change: 1, index: 5 }
purpose 44' selects BIP44, coin type 0' selects the coin, account 2' is the third account, change level 1 is the change branch, and address index 5 is the sixth address.
swap final index to 6: m/44'/0'/2'/1/6
derived address (m/44'/1'/0'/0/0, regtest): mkpZhYtJu2r87Js3pDiWJDmPte2NRZ8bJV
```

(the derived address uses coin type `1'`, the standard "any testnet" coin type, so
it's a `m`/`n` regtest address — same disposable class mnemonic as every other lab.)

## Evidence references

- `src/labs/lab09_bip44.rs` — `decode_bip44_path`, `describe_bip44_path`,
  `with_address_index`, `derive_bip44_address`.
- `tests/lab_09.rs::decodes_every_bip44_level` — `m/44'/0'/2'/1/5` decodes to
  `purpose: 44, coin_type: 0, account: 2, change: 1, index: 5`, matching the struct
  printed above.
- `tests/lab_09.rs::changes_only_the_final_index` — `with_address_index(..., 6)`
  turns `m/44'/0'/2'/1/5` into `m/44'/0'/2'/1/6`, and every other level is untouched
  — matches the "swap final index to 6" line above.

## Explanation

`m/44'/0'/2'/1/5` has five levels, and each one narrows down what you're deriving.
`44'` is the purpose — a fixed constant that just says "this whole subtree follows the
BIP44 convention for organizing accounts and chains." `0'` is the coin type from
SLIP-44 (0 is mainnet Bitcoin; testnet/regtest/signet all conventionally share coin
type `1'`). `2'` is the account — and it's zero-based, so account `0'` is the first
account a wallet creates, account `1'` is the second, and account `2'` is the third,
which is why the description above says "third account" for that level. Treating
accounts as zero-indexed matters mostly for not off-by-one-ing when you're building
UI or docs around it — "account 2" in the path is the third account a user would see
listed.

The apostrophes on the first three levels (`44'`, `0'`, `2'`) mark them as hardened
derivation steps — in BIP32 terms, index + 2^31, though this lab reports the
human-readable index without that offset baked in. Purpose, coin type, and account are
hardened specifically because they sit near the top of the tree and because hardening
requires the parent's private key to derive — so even if an xpub at some level above
these got exposed, an attacker still can't derive into a different account or coin
type from it.

The fourth level, `1` (no apostrophe, so *not* hardened), is the change/receive
branch — by convention `0` means the external/receiving chain (addresses you hand out
to get paid) and `1` means the internal/change chain (addresses the wallet generates
on its own to send change back to itself, so it's not typically shown to anyone).
Because this level is non-hardened, an xpub at the account level can derive both the
`0` and `1` branches, and every address under them, without any private key — exactly
what a watch-only or accounting integration needs.

The fifth level, `5`, is the address index within whichever branch — also zero-based
and non-hardened, so index 5 is the sixth address generated on that branch (indices
0 through 5), and it's the one part of the path a wallet increments every time it
hands out a fresh address.
