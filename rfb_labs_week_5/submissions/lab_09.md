# Lab 09 — BIP44 path decoding

## Commands used

`cargo test --test lab_09`

## Terminal output

```bash
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.26s
     Running tests/lab_09.rs (target/debug/deps/lab_09-2e5d193eb57f896c)

running 4 tests
test changes_only_the_final_index ... ok
test decodes_every_bip44_level ... ok
test explains_zero_based_account_and_chain ... ok
test derives_the_selected_bip44_address ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
``` 

## Evidence references

Code: src/labs/lab09_bip44.rs  
Test: tests/lab_09.rs

## Explanation

### purpose, coin type, account, receive/change, and index.

In a derivation path such as:
`m/84'/0'/0'/0/5`

purpose identifies the wallet standard being used. For example, 44' generally means legacy BIP44 P2PKH addresses, 49' means nested SegWit P2SH-P2WPKH addresses, and 84' means native SegWit P2WPKH addresses.

coin type identifies the blockchain. In common Bitcoin paths, 0' means Bitcoin mainnet, while 1' means Bitcoin testnet.

account separates independent wallet accounts. 0' is usually the first account, 1' the second, and so on. Accounts are commonly hardened.

receive/change identifies the branch of addresses being derived. 0 is the external or receiving branch, used for addresses shown to other people. 1 is the internal or change branch, used for change returned to the wallet.

index identifies an individual address within that branch. In .../0/5, the final 5 means the sixth receiving address, because indexing starts at zero.