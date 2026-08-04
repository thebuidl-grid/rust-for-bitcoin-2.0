# Lab 07 — Confirmation and block membership

## Commands used

cargo build
cargo run --bin lab_07_usage

## Terminal output

TODO: Show the empty mempool, confirmation count, block hash, and TXID in block.

## Evidence references

TODO: Link screenshots or describe the attached evidence.

## Explanation

When a transaction becomes “confirmed,” it’s included in a block and leaves the mempool.
The txid doesn’t change; what changes is its status—confirmations increase to ≥ 1, and its outputs become valid UTXOs on the active chain.
If a reorg happens, it could become unconfirmed again, but normally it stays in place.