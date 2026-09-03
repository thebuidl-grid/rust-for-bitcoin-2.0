# Lab 04 — UTXOs and outpoints

## Commands used

Ran `-rpcwallet=miner listunspent` and `-rpcwallet=miner getbalance` after maturity.

## Terminal output

Selected outpoint `23dddf2a7fc326b70db3fd378a4ee4537cc530af839c68ec0f3401a0650fab2a:0` held 50 BTC, 101 confirmations, script `0014bbee9d26c81c0bbc8ed132f11ab429ca50d7abe4`, and `spendable: true`. Spendable sum and wallet balance were both 50 BTC.

## Evidence references

Captured `listunspent` JSON contains the address, label, amount, confirmations, scriptPubKey, and spendable field.

## Explanation

An outpoint is a unique `txid:vout` coordinate. A wallet balance is a calculated sum of controlled spendable UTXOs, not a single mutable account entry.
