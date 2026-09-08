# Lab 01 — Regtest network inspection

## Commands used

```
cargo test --test lab_01

bitcoin-cli -regtest getblockchaininfo
bitcoin-cli -regtest getblockcount
bitcoin-cli -regtest getbestblockhash
```

*RPCs are the ones issued by `get_chain`, `get_block_height`, and `get_best_block_hash` in `src/labs/lab01_network.rs`, verified against the mocked RPC client in `tests/lab_01.rs`. Run the `bitcoin-cli` lines against your live Polar regtest node to capture the terminal output below.*

## Terminal output

Captured against a live two-node regtest network (`bitcoind-lab-a` / `bitcoind-lab-b`, Bitcoin Core 30.0) at genesis, before any blocks were mined:

```
$ bitcoin-cli -regtest getblockchaininfo
{
  "chain": "regtest",
  "blocks": 0,
  "headers": 0,
  "bestblockhash": "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206",
  "difficulty": 4.656542373906925e-10,
  "chainwork": "0000000000000000000000000000000000000000000000000000000000000002",
  ...
}

$ bitcoin-cli -regtest getblockcount
0

$ bitcoin-cli -regtest getbestblockhash
0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206
```

`chain` = `regtest`, `blocks` (height) = `0`, `bestblockhash` = the regtest genesis block hash — these are exactly the three values `get_chain`, `get_block_height`, and `get_best_block_hash` return.

## Evidence references

Evidence is the live terminal output above, captured directly via `docker exec bitcoind-lab-a bitcoin-cli ...` against a real regtest node (not a screenshot — this session ran headlessly, no Polar GUI was open).

## Explanation

**Bitcoin Core** is the reference implementation of the Bitcoin node software: it validates blocks and transactions, maintains a copy of the blockchain, and exposes an RPC API (`bitcoin-cli`) so other programs can query and control it.

**Regtest** ("regression test mode") is one of Bitcoin Core's network modes, alongside mainnet, testnet, and signet. It's a private, local blockchain that only you control:
- Blocks aren't mined by real proof-of-work competition — you mine them on demand with `generatetoaddress`, instantly, for free.
- Coins have no real-world value, so it's safe to experiment (send funny amounts, break things, reorg the chain) without any risk.
- `getblockchaininfo`'s `chain` field literally says `"regtest"` to confirm which network a node is on.

**Docker** packages Bitcoin Core (and everything it needs to run) into a container — an isolated, reproducible environment. Instead of installing Bitcoin Core directly on your machine, you run it inside a container, which keeps it cleanly separated from the rest of your system and lets you spin up multiple independent nodes side by side (as Lab 10 needs).

**Polar** is a GUI on top of Docker that manages regtest Lightning/Bitcoin networks for you — it creates the containers, wires up the nodes, and gives you buttons to mine blocks and open channels instead of typing raw commands. Under the hood it's running the same `bitcoind` Docker image and the same RPCs used throughout these labs; Polar is a convenience layer, not a different technology.
