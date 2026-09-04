# Lab 09 — BIP44 path decoding

## Commands used

```bash
cargo test --test lab_09 -- --nocapture
```

## Terminal output

```
running 4 tests
test changes_only_the_final_index ... ok
test decodes_every_bip44_level ... ok
test explains_zero_based_account_and_chain ... ok
test derives_the_selected_bip44_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
```

## Evidence references

All four public tests in `tests/lab_09.rs` pass against `src/labs/lab09_bip44.rs`:
`decode_bip44_path("m/44'/0'/2'/1/5")` decodes to `{purpose: 44, coin_type: 0, account: 2,
change: 1, index: 5}`; `describe_bip44_path` produces English text containing "third account",
"change", and "sixth address" for that same decoded path; `with_address_index` replaces only the
final index (`.../1/5` → `.../1/6`); `derive_bip44_address` derives a deterministic
`m`/`n`-prefixed regtest P2PKH address from `m/44'/1'/0'/0/0` on the public test mnemonic.

## Explanation

**Zero-based account and address indexes:** BIP44's `account'` and `address_index` levels both
start counting at `0`, not `1`. `decode_bip44_path` on `m/44'/0'/2'/1/5` reports `account: 2`, and
`describe_bip44_path` renders that as "the **third** account" — index `0` is the first account,
index `1` the second, index `2` the third. The same logic applies to `index: 5`, rendered as "the
**sixth** address" — this is a very common off-by-one trap when reading raw derivation paths, so
this lab's explanation deliberately spells out the ordinal alongside the raw zero-based number.

**Hardened apostrophes:** the `'` (or `h`) suffix on `purpose'`, `coin_type'`, and `account'` marks
those three levels as *hardened* derivation — required, per BIP44, because those levels sit above
the receive/change split and must not be derivable from a leaked xpub plus a sibling xpriv (see
Lab 08's chain-code explanation). `change` and `index` are deliberately left non-hardened, because
non-hardened derivation is what makes an xpub at the `account` level usable as a watch-only key
that can still enumerate every receiving and change address underneath it.

**Receive/change branch:** the `change` level is a binary switch, not an arbitrary index — `0`
means the **receive** (external) branch, addresses meant to be handed out to other people to pay
you, and `1` means the **change** (internal) branch, addresses the wallet generates for itself to
receive its own leftover change from a transaction, which should never be shared externally.
`describe_bip44_path` reports `change: 1` in the decoded path as the "change chain," matching this
convention, and `derive_bip44_address` in this lab always derives on the receive branch
(`.../0/index`) since that's the address a wallet would actually hand out to receive a payment.
