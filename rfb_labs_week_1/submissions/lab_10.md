# Lab 10 — Competing branches and reorganization

## Commands used

```bash
cargo test --test lab_10
bitcoin-cli -regtest getblockchaininfo
bitcoin-cli -regtest disconnectnode <peer-address>
bitcoin-cli -regtest generatetoaddress 2 <node-a-miner-address>
bitcoin-cli -regtest generatetoaddress 4 <node-b-miner-address>
bitcoin-cli -regtest getblockchaininfo
bitcoin-cli -regtest addnode <peer-address> onetry
bitcoin-cli -regtest getblockchaininfo
```

## Terminal output

Both nodes started from the same common tip. While disconnected, Node A produced a shorter private branch and Node B produced a longer private branch with greater accumulated chainwork. After reconnection and synchronization, both nodes reported the same final height, best block hash, and chainwork.

## Evidence references

Evidence is the Lab 10 test run and the two-node chain-tip transcript showing common pre-split tip, competing private tips, chainwork values, and final convergence after reconnection.

## Explanation

The shorter branch became stale because the network converged on the valid branch with the greatest accumulated proof of work. A reorganization is the node replacing part of its active chain with a stronger valid branch. Nodes do not choose by miner identity, first arrival, or social claim; they choose the valid chain with the most accumulated work.
