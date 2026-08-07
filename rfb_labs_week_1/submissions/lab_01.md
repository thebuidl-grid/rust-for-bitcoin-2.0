# Lab 01 — Regtest network inspection

## Commands used

TODO: List the Rust command and Bitcoin Core RPCs you ran.
```bash
# Verify Bitcoin Core node info in Polar
bitcoin-cli -regtest getblockchaininfo
bitcoin-cli -regtest getblockcount
bitcoin-cli -regtest getbestblockhash

# Run cargo test suite
cargo test --test lab_01
```
## Terminal output

TODO: Record chain, block height, and best-block hash.
elsuraj@El-suraj:~/rust-for-bitcoin-2.0/rfb_labs_week_1$ cargo test --test lab_01
   Compiling rfb-labs-week-1 v0.1.0 (/home/elsuraj/rust-for-bitcoin-2.0/rfb_labs_week_1)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.55s
     Running tests/lab_01.rs (target/debug/deps/lab_01-05d249e955f81e91)

running 4 tests
test reads_best_block_hash ... ok
test builds_verified_network_snapshot ... ok
test reads_block_height ... ok
test reads_regtest_chain ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


## Evidence references

TODO: Link screenshots or describe the attached evidence.

## Explanation

TODO: Explain Polar, Docker, Bitcoin Core, and regtest in your own words.
1. Polar: A graphical desktop orchestration tool designed to easily deploy, connect, and visualize local Bitcoin and Lightning Network topologies. It automates Docker container setup and RPC credentials management.

2. Docker: A containerization platform that isolates individual node instances (bitcoind, lnd, cln) into lightweight, repeatable virtual environments without interfering with host operating system dependencies.

3. Bitcoin Core: The reference full node daemon (bitcoind) that implements consensus validation, maintains the UTXO database, indexes the blockchain, and provides a JSON-RPC interface (bitcoin-cli).

4. regtest (Regression Test): A local, private testing network where difficulty is set to near zero, allowing blocks to be instantly mined on demand via RPC commands (e.g., generatetoaddress).