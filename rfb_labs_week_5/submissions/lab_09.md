# Lab 09 — BIP44 path decoding

## Commands used

```bash
cargo test --test lab_09 -- --nocapture
cargo run --example labs_demo
```

## Terminal output

```text
$ cargo test --test lab_09 -- --nocapture
running 4 tests
test decodes_every_bip44_level ... ok
test changes_only_the_final_index ... ok
test explains_zero_based_account_and_chain ... ok
test derives_the_selected_bip44_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

```text
$ cargo run --example labs_demo   (Lab 09 section, public test mnemonic, regtest)
decode_bip44_path(m/44'/0'/2'/1/5) = Bip44PathInfo { purpose: 44, coin_type: 0, account: 2, change: 1, index: 5 }
describe_bip44_path = "BIP44 account-structured path: coin type 0, third account, change branch, sixth address."
with_address_index(..., 6) = Ok("m/44'/0'/2'/1/6")
derive_bip44_address(m/44'/1'/0'/0/0, Regtest) = Ok("mkpZhYtJu2r87Js3pDiWJDmPte2NRZ8bJV")
```

## Evidence references

- Implementation: [`src/labs/lab09_bip44.rs`](../src/labs/lab09_bip44.rs)
- Public test suite: [`tests/lab_09.rs`](../tests/lab_09.rs) — 4/4 passing, logged in
  [`grading/logs/lab_09.log`](../grading/logs/lab_09.log).
- `with_address_index` changes only the final component (`.../1/5` → `.../1/6`),
  leaving `m/44'/0'/2'/1` untouched.
- `derive_bip44_address` output `mkpZhYtJu2r87Js3pDiWJDmPte2NRZ8bJV` starts with `m`,
  the regtest/testnet P2PKH prefix, and is derived from the public test mnemonic only.

## Explanation

`m/44'/0'/2'/1/5` decodes level by level under BIP44's fixed five-level structure:

- **`44'` — purpose.** Hardened, fixed at 44 for every BIP44 wallet; it is what
  identifies this whole subtree as following the BIP44 account-structure convention
  (as opposed to, say, BIP49's `49'` or BIP84's `84'` used in Lab 10).
- **`0'` — coin type.** Hardened; selects which coin's registered SLIP-44 index this
  subtree belongs to (`0'` for Bitcoin mainnet, `1'` for Bitcoin's test networks —
  which is why `derive_bip44_address` above uses `1'` on regtest).
- **`2'` — account.** Hardened; BIP44 accounts are **zero-based**, so account index `2`
  is the *third* account a wallet has created (`0` → first, `1` → second, `2` → third)
  — separate spending/labeling buckets under one wallet, e.g. "personal" vs. "business."
- **`1` — change.** Normal (not hardened); this is the branch selector, conventionally
  `0` for the **receive** chain (addresses handed out to be paid) and `1` for the
  **change** chain (addresses a wallet sends its own change back to, keeping change
  outputs visibly separate from outputs the user actually shared).
- **`5` — address index.** Normal; also zero-based, so index `5` is the *sixth* address
  generated on that branch.

The apostrophe (`'`, sometimes written `h`) marks a level as **hardened**, meaning (per
Lab 08) it can only be derived from a private key, not from a parent xpub — which is
exactly why BIP44 hardens `purpose'`/`coin_type'`/`account'` (protecting the account
tree from a leaked xpub-plus-one-child-privkey attack) while leaving `change`/`index`
normal (so a single account-level xpub can still generate every receive and change
address, watch-only, without exposing anything above it).
