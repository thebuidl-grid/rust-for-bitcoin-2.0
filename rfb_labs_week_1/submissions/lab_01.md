# Lab 01 — Regtest network inspection

## Commands used

docker exec polar-n3-backend1 bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getblockchaininfo
cargo test --test lab_01
cargo run --example lab01_check


## Terminal output

TODO: Record chain, block height, and best-block hash.

```NetworkSnapshot {
    chain: "regtest",
    block_height: 1,
    best_block_hash: "4a9529e0ad6df87aa819bfd3da8c05af0f24baf7c7ea260f2316636d57472463",
}
```

## Evidence references

TODO: Link screenshots or describe the attached evidence.
https://drive.google.com/drive/folders/1mP1ycuASg9SOfhFiHK00MdBMmprZZjQp?usp=drive_link


## Explanation

TODO: Explain Polar, Docker, Bitcoin Core, and regtest in your own words.

Bitcoin Core is the actual software — a full implementation of the Bitcoin protocol. It maintains a copy of the blockchain, validates transactions and blocks against consensus rules, relays data to peers, and exposes an RPC interface (bitcoin-cli talks to this) so other programs — like the Rust code — can query and control it.

regtest ("regression test mode") is one of several networks Bitcoin Core can run on with fake coins but still shared with other users. Regtest is private and local: only you control it, blocks are mined instantly on command (no real proof-of-work race against the world), and coins have no value. 

Docker packages Bitcoin Core (and its exact dependencies/config) into an isolated container, so we get a clean, disposable, reproducible node without installing anything directly on my machine.

Polar is the orchestration layer on top: a GUI that generates the Docker Compose setup, launches the containers with the right regtest flags/ports/credentials and gives a visual network map — so we don't have to hand-write Docker config to get a Bitcoin Core regtest node running.