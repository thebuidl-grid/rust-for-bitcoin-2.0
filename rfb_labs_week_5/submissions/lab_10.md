# Lab 10 — Deterministic recovery across address families

## Commands used

`cargo test --test lab_10`

## Terminal output

```bash
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
     Running tests/lab_10.rs (target/debug/deps/lab_10-a39b1ec0ed495d43)

running 4 tests
test identical_recovery_inputs_repeat ... ok
test changing_only_the_index_changes_the_address ... ok
test format_selection_changes_the_lock_target ... ok
test derives_three_regtest_address_families ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
``` 

## Evidence references

Code: src/labs/lab10_recovery.rs  
Test: tests/lab_10.rs

## Explanation

### how recovery inputs and derivation conventions reproduce a wallet.

Wallet recovery uses the original mnemonic, optional passphrase, network, and derivation conventions to regenerate the same deterministic key hierarchy.

The mnemonic is converted into a seed. If a passphrase was used, it must be entered exactly.
The path conventions define how to interpret each level: purpose, coin type, account, receiving or change branch, and address index. The wallet derives the same private key and address at that path every time.

Recovery software usually scans address indexes starting from zero on both the receiving branch and change branch. It looks for transaction history or unused-address gaps to determine which portions of the wallet were used.