# Lab 01 — Regtest network inspection

## Commands used

TODO: List the Rust command and Bitcoin Core RPCs you ran.
```
cargo test --test lab_01
bitcoind -regtest -daemon
bcli getblockchaininfo
```


## Terminal output
TODO: Record chain, block height, and best-block hash.

<img width="824" height="183" alt="Screenshot 2026-07-31 at 22 37 24" src="https://github.com/user-attachments/assets/68bc09ca-8791-404b-9ebb-c6dddc3d5099" />

<img width="661" height="271" alt="Screenshot 2026-07-31 at 05 21 16" src="https://github.com/user-attachments/assets/6f9e3d93-9c52-45bb-80c8-9a7f40615341" />

chain : regtest

block height : 0

best-block hash : 0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206

## Evidence references

TODO: Link screenshots or describe the attached evidence.

From the screenshot above, all the tests passed, also, we can see the chain on which we are working on(regtest), along with other information like block height, block hash, difficulty, taget and many more.

## Explanation

TODO: Explain Polar, Docker, Bitcoin Core, and regtest in your own words.

Polar is a desktop app that spins up multiple Dockerized Bitcoin Core nodes with a visual interface, so one can build and observe multi-node networks, forks, and other things.

Docker is a tool for packaging software (like Bitcoin Core) into isolated, portable "containers" that run consistently regardless of the host machine's setup, Polar uses Docker 

Bitcoin Core is the most widely used software implementation of the Bitcoin protocol — it's what actually runs a Bitcoin node, validates transactions and blocks, and exposes the RPC commands (like `getblockcount` or `sendtoaddress`)

Regtest ("regression test mode") is a special local-only network built into Bitcoin Core that lets one mine blocks instantly on command, create money out of thin air, and fully control the chain


