\# Lab 01 — Regtest network inspection



\## Commands used



Commands executed:



```bash

cargo test --test lab\_01

cargo test



Bitcoin Core RPC calls used by the lab:



getblockchaininfo

getbestblockhash

getblockcount

Terminal output



The Lab 01 test suite completed successfully.



Finished test profile \[unoptimized + debuginfo] target(s) in 0.08s

Running tests\\lab\_01.rs



running 4 tests



test reads\_best\_block\_hash ... ok

test builds\_verified\_network\_snapshot ... ok

test reads\_block\_height ... ok

test reads\_regtest\_chain ... ok



test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out



The tests confirmed that the Rust application successfully communicated with the Bitcoin Core regtest node and retrieved:



The active blockchain network information.

The current block height.

The best block hash.

A verified network snapshot.

Evidence references



Evidence collected:



Terminal output showing cargo test --test lab\_01 passing.

Screenshot reference: lab01-test-success.png.

Explanation



Polar is a development tool used to create and manage local Bitcoin networks. It can launch Bitcoin Core nodes in Docker containers and connect them together in a controlled testing environment.



Docker provides isolated containers that allow Bitcoin Core services to run without affecting the host machine.



Bitcoin Core is the reference Bitcoin node software. It provides RPC commands that applications can use to communicate with the blockchain, retrieve information, manage wallets, and interact with transactions.



Regtest (regression test network) is a private Bitcoin network designed for development and testing. Unlike mainnet, regtest allows developers to create blocks instantly and control the blockchain environment.



This makes regtest useful for testing wallet operations, mining, confirmations, transactions, and other Bitcoin functionality without using real bitcoin.

