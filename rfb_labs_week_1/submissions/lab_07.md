# Lab 07 — Confirmation and block membership

## Commands used

cargo run --example lab07

Underlying bitcoin-cli RPCs invoked:
- generatetoaddress 1 bcrt1qsfqwvhu2yn2ghu5yj2dsajdck38gykmk0nq7cn
- getrawmempool
- gettransaction a9e5849b95b19d9c08218953eeb0475c75b8b856f5838615bd37f37f6056647b   (wallet: receiver)
- getblock 16e63e070ff88f883247a9ee8208206b73348a8f401ab5b1d108d9884adea592 1


## Terminal output

txid:                    a9e5849b95b19d9c08218953eeb0475c75b8b856f5838615bd37f37f6056647b
block_hash:              16e63e070ff88f883247a9ee8208206b73348a8f401ab5b1d108d9884adea592
confirmations:           2
mempool_is_empty:        true
transaction_is_in_block: true

## Evidence references

mempool_is_empty:        true
transaction_is_in_block: true

## Explanation

The location/status: the transaction moves out of individual nodes' local, temporary mempools and into the block. also, the node's UTXO set gets updated. also, double spend risk drops, and confirmation count starts incrementing.