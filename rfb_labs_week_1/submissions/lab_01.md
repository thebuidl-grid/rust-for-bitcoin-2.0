# Lab 01 — Regtest network inspection

## Commands used


docker exec polar-bitcoin bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getblockchaininfo


## Terminal output

{
  "height": 101,
  "hash": "7368ad4272240cf3f9bd35ea40ab623b6abdb52765a862170951e5e735a84180"
}

## Evidence references


https://drive.google.com/drive/folders/1HvmkTC2bazkXgBELjgbLaaW8grJQgF9h?usp=sharing

## Explanation


Bitcoin Core is the actual software — a full implementation of the Bitcoin protocol. It maintains a copy of the blockchain, validates transactions and blocks against consensus rules, relays data to peers, and exposes an RPC interface (bitcoin-cli talks to this) so other programs — like the Rust code — can query and control it.

regtest ("regression test mode") is one of several networks Bitcoin Core can run on with fake coins but still shared with other users. Regtest is private and local: only you control it, blocks are mined instantly on command (no real proof-of-work race against the world), and coins have no value. 

Docker packages Bitcoin Core (and its exact dependencies/config) into an isolated container, so we get a clean, disposable, reproducible node without installing anything directly on my machine.

Polar is the orchestration layer on top: a GUI that generates the Docker Compose setup, launches the containers with the right regtest flags/ports/credentials and gives a visual network map — so we don't have to hand-write Docker config to get a Bitcoin Core regtest node running.