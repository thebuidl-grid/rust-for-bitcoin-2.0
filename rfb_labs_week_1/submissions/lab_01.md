# Lab 01 — Regtest network inspection

## Commands used

Ran `cargo test --test lab_01` and, in Polar Node A, `bitcoin-cli -regtest getblockchaininfo`, `getblockcount`, and `getbestblockhash`.

## Terminal output

Node A reported `chain: regtest`. The reset test chain began at height 0; both Polar Bitcoin Core nodes initially shared its same tip.

## Evidence references

Terminal JSON was captured from running container `polar-n1-backend1` (`polarlightning/bitcoind:30.0`) in Polar.

## Explanation

Polar builds the local topology, Docker isolates node processes, and Bitcoin Core validates blocks and exposes RPC. Regtest is a private chain with valueless coins and on-demand mining, so this work cannot affect mainnet.
