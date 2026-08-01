# Lab 02 — Wallets and addresses

## Commands used

```bash
cargo test --test lab_02
bitcoin-cli -regtest createwallet miner
bitcoin-cli -regtest createwallet receiver
bitcoin-cli -regtest listwallets
bitcoin-cli -regtest -rpcwallet=miner getnewaddress mining
bitcoin-cli -regtest -rpcwallet=receiver getnewaddress classmate
bitcoin-cli -regtest -rpcwallet=miner getaddressinfo <miner-address>
bitcoin-cli -regtest -rpcwallet=receiver getaddressinfo <receiver-address>
```

## Terminal output

The loaded wallet list contained `miner` and `receiver`. The generated addresses used the regtest `bcrt1` prefix. `getaddressinfo` returned `ismine=true` only when the address was checked in the wallet that created it.

## Evidence references

Evidence is the Lab 02 test run and the wallet RPC transcript showing wallet creation, loaded wallet names, generated labelled addresses, and ownership checks.

## Explanation

Wallet RPCs are scoped. `-rpcwallet=miner` means the call is evaluated using the miner wallet's keys, labels, transactions, and UTXOs. A wrong wallet context can make a valid address appear unrelated, hide relevant transactions, or attempt to spend coins from a wallet that does not control them.
