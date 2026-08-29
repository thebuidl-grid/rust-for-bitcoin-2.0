# Lab 10 — Deterministic recovery across address families

## Commands used

```
cargo test --test lab_10 -- --nocapture
```

## Terminal output

```
running 4 tests
test format_selection_changes_the_lock_target ... ok
test derives_three_regtest_address_families ... ok
test changing_only_the_index_changes_the_address ... ok
test identical_recovery_inputs_repeat ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
```

## Evidence references

Deriving account 0, index 0 on regtest from the public class mnemonic with an empty
passphrase across all three branches:

```
DerivedAddressSet {
    bip44_p2pkh:       "mkpZhYtJu2r87Js3pDiWJDmPte2NRZ8bJV"        (m/44'/1'/0'/0/0)
    bip49_p2sh_p2wpkh: "2Mww8dCYPUpKHofjgcXcBCEGmniw9CoaiD2"        (m/49'/1'/0'/0/0)
    bip84_p2wpkh:      "bcrt1q6rz28mcfaxtmd6v789l9rrlrusdprr9pz3cppk" (m/84'/1'/0'/0/0)
}
```

Each address's prefix matches its script family and the regtest network exactly:
`m`/`n` for legacy P2PKH, `2` for P2SH-wrapped SegWit, `bcrt1q` for native P2WPKH.

Deriving `m/84'/1'/0'/0/0` twice with the same mnemonic and passphrase (`"class"`)
produced the same address both times (`recovery_is_repeatable` → `true`). Deriving index
`0` and index `1` on the same branch produced two different addresses
(`changing_index_changes_address` → `true`). Deriving `m/44'/1'/0'/0/0` as P2PKH versus
P2WPKH produced two different addresses from the identical key material, confirming the
script family — not just the key — determines the final address.

## Explanation

Given the same entropy source (mnemonic + passphrase), BIP32 derivation is a pure
deterministic function of the derivation path: HMAC-SHA512 over the same chain code, same
parent key, and same child index always produces the same child key, with no randomness
involved anywhere in the process. That's why deriving the same path twice reproduces the
same key and therefore the same address every time. But recovering *the same wallet* a
user actually had requires more than reproducing keys — it also requires reproducing the
conventions layered on top of the raw key: which BIP44/49/84 purpose branch was used,
which coin type and account, and which script template (P2PKH, P2SH-P2WPKH, or P2WPKH)
that key was locked into an address with. The same private key produces a completely
different, mutually non-interoperable address under each script family, so a recovery
tool that reproduces the right keys but assumes the wrong path or script convention will
silently generate a wallet that never sees the original funds, even though the underlying
cryptographic derivation was completely correct.
