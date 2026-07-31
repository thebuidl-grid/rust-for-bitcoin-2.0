# Lab 01 — Regtest network inspection

## Commands used

cargo run --example lab01
getblockchaininfo
getblockcount
getbestblockhash

## Terminal output

chain:            regtest
block_height:     1
best_block_hash:  7590e2ee7f48d2d8d39e9021518059a814c0b50ed86b816b05612a76adbc3421

## Evidence references

- Polar network running: ../evidence/lab01/polar-screenshot.png
- Node info panel confirming regtest: ../evidence/lab01/node-info.png
## Explanation

regtest lets you mine blocks instantly on demand, it keeps you fully isolated from the real network state, and lets you use fake coins freely for testing.

Bitcoin Core validates blocks and transactions against consensus rules, it maintains the local chain state and exposes an RPC interface that Rust can call.

Polar is the mastermind that tells Docker which containers to launch, it wires up the RPC ports and credentials between nodes.

Docker provides isolated environment for Bitcoin Core instances to run in, so Polar does not have to risk conflicts between nodes.

Polar instructs Docker to launch a container, then the container runs a Bitcoin Core node configured for regtest, and the Rust code talks to the node's RPC interface to submit transactions, query state, or trigger block generation.