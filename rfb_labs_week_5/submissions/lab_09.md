# Lab 09 — BIP44 path decoding

## Commands used

```shell
test@pop-os:~/Desktop/rust/rust-for-bitcoin-2.0/rfb_labs_week_5$ cargo test --test lab_09
```

## Terminal output

```terminaloutput
running 4 tests
test changes_only_the_final_index ... ok
test decodes_every_bip44_level ... ok
test explains_zero_based_account_and_chain ... ok
test derives_the_selected_bip44_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

`decode_bip44_path("m/44'/0'/2'/1/5")` decodes the five levels as `purpose: 44`,
`coin_type: 0`, `account: 2`, `change: 1`, `index: 5`.
`with_address_index("m/44'/0'/2'/1/5", 6)` yields `m/44'/0'/2'/1/6` — only the final
child changes. `describe_bip44_path` renders it as a description containing
"third account", "change chain", and "sixth address". From the public test mnemonic,
`derive_bip44_address` at `m/44'/1'/0'/0/0` on regtest returns a deterministic
`m...`/`n...` (P2PKH) address (disposable input; not reproduced as secret material).

## Evidence references

```
Code: src/labs/lab09_bip44.rs
Test: tests/lab_09.rs
```

## Explanation

A BIP44 path `m / purpose' / coin_type' / account' / change / index` has five levels,
and each is zero-based.

- **Purpose** (`44'`) names the BIP that defines the layout. BIP44 uses hardened `44'`.
- **Coin type** selects the blockchain (e.g. `0` for Bitcoin, `1` for any testnet/
  regtest network).
- **Account** partitions keys into independent accounts. Because it is zero-based,
  account `2` is the **third** account.
- **Change** is the receive/change branch: `0` = receiving (external) addresses,
  `1` = change (internal). This is why `change: 1` is described as the "change chain".
- **Index** is the address position within that branch, also zero-based, so index `5`
  is the **sixth** address.

The **apostrophe** (`'`, e.g. `44'`) marks a **hardened** step. Hardened steps use a
separate child-number range (index plus `2^31`) and are derived only from the parent
private key, so they cannot be derived from a parent xpub. The final `change` and
`index` levels are conventionally **not** hardened.

BIP44 exists so that a single seed deterministically maps to many addresses: change
only the final index and the next address in the same receive branch is produced
(`with_address_index`), which is how wallets keep a known, reproducible address order
across restores.
