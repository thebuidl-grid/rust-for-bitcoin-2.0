# Lab 01 — Regtest network inspection

## Commands used

Rust: `cargo run --example run` (calls `inspect_network`, which internally calls):
- `getblockchaininfo` — via `get_chain`
- `getblockcount` — via `get_block_height`
- `getbestblockhash` — via `get_best_block_hash`

## Terminal output

=== Lab 01: network ===
NetworkSnapshot {
chain: "regtest",
block_height: 1,
best_block_hash: "7147efb264c95130dee29346ed2292bd1e0a9f9b2b6308fd2abeb2847a9d2a7a",
}

## Evidence references

Screenshot: `evidence/lab01.png`

## Explanation

Docker is the container runtime that actually runs the Bitcoin Core node — it packages the software and its dependencies into an isolated environment so it doesn't need to be installed directly on my machine.

Polar is a GUI on top of Docker built specifically for local Bitcoin/Lightning development. Instead of manually writing Docker configs and `bitcoin.conf` files, Polar lets me spin up a Bitcoin Core node with a few clicks and gives me the exact RPC connection details (host, port, username, password) needed to talk to it.

Bitcoin Core is the actual node software — the program that validates blocks and transactions, maintains the blockchain, and exposes an RPC interface (`bitcoin-cli`) that other programs, including my Rust code, can call to interact with it.

Regtest ("regression test") is one of Bitcoin Core's network modes, alongside mainnet and testnet. It's a fully private, local-only blockchain that I fully control — no other peers, no real money, no real proof-of-work difficulty.