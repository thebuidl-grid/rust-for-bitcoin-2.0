# Lab 09 — BIP44 path decoding

## Commands used

```bash
cargo test --test lab_09
bash grader/grade.sh
```

## Terminal output

running 4 tests

test decodes_every_bip44_level ... ok

test changes_only_the_final_index ... ok

test explains_zero_based_account_and_chain ... ok

test derives_the_selected_bip44_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

## Evidence references

Example: `m/44'/0'/2'/1/5` decodes to purpose 44, coin type 0, account 2,
change 1, index 5 — described as "the third account, the change branch, the
sixth address." Replacing only the index gives `m/44'/0'/2'/1/6`. Deriving
at `m/44'/1'/0'/0/0` (testnet/regtest coin type) on the public test
mnemonic produces a disposable regtest P2PKH address starting with `m` or
`n`, deterministically reproducible from the same inputs.

All four public tests in `tests/lab_09.rs` pass, using only the published
public BIP39 test mnemonic:
- Decoding every level of `m/44'/0'/2'/1/5` into its five components
  (`decodes_every_bip44_level`)
- Producing a human-readable explanation naming the third account, the
  change branch, and the sixth address (`explains_zero_based_account_and_chain`)
- Replacing only the final index while preserving purpose, coin type,
  account, and branch (`changes_only_the_final_index`)
- Deriving a deterministic, disposable regtest P2PKH address from the
  public test mnemonic at a full BIP44 path (`derives_the_selected_bip44_address`)

## Explanation

A BIP44 path has five fixed levels: `m / purpose' / coin_type' / account' /
change / index`.

- **Purpose** is always `44'` for this standard — it identifies the path as
  following the BIP44 convention specifically, as opposed to BIP49 or
  BIP84.
- **Coin type** selects which cryptocurrency the rest of the path applies
  to (`0'` for Bitcoin mainnet, `1'` for Bitcoin testnet/regtest in this
  lab), letting a single seed support multiple coins without key reuse
  across them.
- **Account** is a zero-based index for separating a user's funds into
  independent, logically distinct wallets under one seed — account `0` is
  the *first* account, account `2` is the *third*, even though the number
  itself reads "2." This lets someone organize funds (for example,
  personal vs. business) while still recovering everything from one
  mnemonic.
- **Change** selects between two branches: `0` is the receive branch
  (addresses given out to receive payments), `1` is the change branch
  (addresses a wallet generates internally to hold leftover value when
  spending). This distinction exists so external and internal address
  generation can be scanned and audited separately.
- **Index** is again zero-based and selects which specific address within
  the chosen account/branch to derive — index `0` is the first address,
  index `5` is the sixth.

The **hardened apostrophes** on purpose, coin type, and account (but not
change or index) mean those three levels can only ever be derived from a
parent *xpriv*, never from an xpub — this deliberately prevents a
watch-only xpub at the account level or above from being used to derive
sibling accounts or coin types, and protects against the parent-key-leak
weakness that hardened derivation exists to close (as covered in Lab 08).
Change and index stay non-hardened specifically so a single account-level
xpub can be handed to watch-only software, letting it generate every
receive and change address without ever holding signing capability.

