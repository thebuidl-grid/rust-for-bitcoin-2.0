# Lab 10 — Deterministic recovery across address families

## Commands used

```shell
test@pop-os:~/Desktop/rust/rust-for-bitcoin-2.0/rfb_labs_week_5$ cargo test --test lab_10
```

## Terminal output

```terminaloutput
running 4 tests
test format_selection_changes_the_lock_target ... ok
test changing_only_the_index_changes_the_address ... ok
test identical_recovery_inputs_repeat ... ok
test derives_three_regtest_address_families ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

For the public test mnemonic with an empty passphrase, account 0, index 0 on regtest,
`derive_address_set` returns three distinct families:

| Standard | Path | Script family | Regtest prefix |
|----------|------|---------------|----------------|
| BIP44 | `m/44'/1'/0'/0/0` | P2PKH | `m`/`n` |
| BIP49 | `m/49'/1'/0'/0/0` | P2SH-wrapped P2WPKH | `2` |
| BIP84 | `m/84'/1'/0'/0/0` | native P2WPKH | `bcrt1q` |

`recovery_is_repeatable` returns true (identical inputs reproduce the same address),
and `changing_index_changes_address` returns true (path `.../0/0` → `.../0/1` yields
a different address).

## Evidence references

```
Code: src/labs/lab10_recovery.rs
Test: tests/lab_10.rs
```

## Explanation

Recovery is **deterministic**: the same seed and derivation conventions always
produce the same wallet. Every address here flows from one mnemonic through the same
BIP32 path logic, so the same inputs (`mnemonic + passphrase + path + network`)
always yield the same child key and, hence, the same address —
`recovery_is_repeatable` demonstrates exactly this.

Restoring a wallet is not automatic, though — it also depends on **path and script
conventions**. The three standards exist to keep deterministic addresses for
different capabilities:

- **BIP44** uses the `44'` purpose to derive P2PKH (legacy) addresses.
- **BIP49** uses `49'` for P2SH-wrapped SegWit (P2SH-P2WPKH), a `3...` address that
  old Base58 wallets can still pay.
- **BIP84** uses `84'` for native SegWit (P2WPKH), a `bcrt1q`/`bc1q` address.

All three take the same account and index and stay on the same receive branch
(`change = 0`). Because the paths differ, changing only the final `index` moves to
the next address within that branch, and any change in mnemonic, passphrase, account,
path, or script family selects a different address. To restore the *original*
addresses, a wallet must know not only the seed but also which derivation scheme
(BIP44/49/84) and which exact paths were used — confirming that recovery depends on
both the recovery inputs and the standardization those inputs are interpreted under.
