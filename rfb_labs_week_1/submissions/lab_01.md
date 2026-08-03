# Lab 01 — Regtest network inspection

## Commands used

`cargo test --test lab_01`

Direct node check:

`docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie getblockchaininfo`

`docker exec polar-n1-backend1 bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie getblockcount`

RPCs used by the Rust implementation:

- `getblockchaininfo`
- `getblockcount`
- `getbestblockhash`

## Terminal output

The public lab test passed and the live Polar node check returned:

- chain: `regtest`
- block height: `103`
- best-block hash: `1403a86d956b950caac8d703ac4eb5d517d855b89fc6d348f55a6570aa6e5f9f`

The test run finished successfully with all 4 lab 1 tests passing.

## Evidence references

Screenshot of the Polar Bitcoin Core node terminal or Docker terminal showing the successful `bitcoin-cli` calls against `polar-n1-backend1`. The screenshot should show `regtest`, block height `103`, and the best-block hash above.

## Explanation

Polar is the app that creates and manages the Bitcoin lab environment. Docker is the container system Polar uses to run the node software. Bitcoin Core is the actual Bitcoin node that answers RPC calls and tracks the chain. Regtest is a private local testing network where blocks can be mined instantly, so it is safe for labs and does not use real Bitcoin.

In Rust, this lab is about turning raw RPC text into typed values. `get_chain` reads JSON from `getblockchaininfo`, `get_block_height` parses the block count as a `u64`, and `get_best_block_hash` returns the hash as a `String`. `inspect_network` combines those values and checks that the node is really on `regtest`, which is a good example of Rust’s type safety and `Result`-based error handling.
