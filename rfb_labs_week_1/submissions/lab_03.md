# Lab 03 — Coinbase maturity

## Commands used

```bash
cargo test --test lab_03
bitcoin-cli -regtest generatetoaddress 1 <miner-address>
bitcoin-cli -regtest getblockcount
bitcoin-cli -regtest -rpcwallet=miner getbalances
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress <receiver-address> 1
bitcoin-cli -regtest generatetoaddress 100 <miner-address>
bitcoin-cli -regtest getblockcount
bitcoin-cli -regtest -rpcwallet=miner getbalances
```

## Terminal output

At height 1, the miner wallet reported the first 50 BTC coinbase reward as immature and the premature 1 BTC spend failed with an insufficient funds RPC error. After 100 more blocks, the chain reached height 101; the first reward became trusted while later coinbase rewards remained immature.

## Evidence references

Evidence is the Lab 03 test run and the RPC transcript containing the two `getbalances` snapshots, the rejected premature spend, and the final height of 101.

## Explanation

Bitcoin Core enforces coinbase maturity: a coinbase output cannot be spent until it has 100 confirmations. On a fresh regtest chain, mining block 1 creates the first coinbase, and mining through height 101 gives that output 101 blocks of depth, making it spendable. The later rewards are still young, so they stay in the immature balance.
