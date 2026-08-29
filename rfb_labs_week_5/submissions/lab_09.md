# Lab 09 — BIP44 path decoding

## Commands used

```
cargo test --test lab_09 -- --nocapture
```

## Terminal output

```
running 4 tests
test changes_only_the_final_index ... ok
test decodes_every_bip44_level ... ok
test explains_zero_based_account_and_chain ... ok
test derives_the_selected_bip44_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
```

## Evidence references

Decoding `m/44'/0'/2'/1/5`:

```
Bip44PathInfo { purpose: 44, coin_type: 0, account: 2, change: 1, index: 5 }

description: "purpose 44' selects BIP44, coin type 0' selects the coin, account 2' is the
third account (zero-based), the change branch is index 1 (change), and address index 5
is the sixth address (zero-based)"
```

`with_address_index("m/44'/0'/2'/1/5", 6)` returned `m/44'/0'/2'/1/6`, changing only the
final index and preserving every other level exactly.

Deriving the address selected by `m/44'/1'/0'/0/0` on regtest with the public class
mnemonic produced `mkpZhYtJu2r87Js3pDiWJDmPte2NRZ8bJV`, a legacy P2PKH-style address
(`m`/`n` prefix, as expected for testnet/regtest), reproduced identically on a second call
with the same inputs.

## Explanation

BIP44 paths follow `m / purpose' / coin_type' / account' / change / address_index`.
`purpose'` is always `44'`, marking this as a BIP44 derivation. `coin_type'` selects
which cryptocurrency's registered SLIP-44 number to use (0' for Bitcoin mainnet, 1' for
any testnet/regtest/signet by convention). `account'` and `address_index` are both
zero-based: account `2'` is the *third* account a wallet has created, not the second, and
index `5` is the *sixth* address on that branch — this trips people up because the
apostrophe-marked numbers read like ordinals but count from zero like array indices. The
apostrophe denotes hardened derivation (child index + 2^31), used for `purpose`,
`coin_type`, and `account` specifically so that a leaked account-level xpub can never be
used to walk back up and compromise the wallet's master or sibling-account keys — hardened
steps require the parent's private key, which an xpub never exposes. `change` is not
hardened and selects one of two conventional branches: `0` for receive addresses (given
out to be paid), `1` for change addresses (used internally by the wallet to return
leftover value from a transaction to itself).
