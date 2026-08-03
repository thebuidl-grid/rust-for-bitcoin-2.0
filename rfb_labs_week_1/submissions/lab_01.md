# Lab 01 — Regtest network inspection

## Commands used

```bash
# Polar: created network "Week 1 Bitcoin Fundamentals" with one Bitcoin Core node (regtest)

# Rust tests
cd rfb_labs_week_1
cargo test --test lab_01

# Bitcoin Core RPCs (inside Polar node terminal)
bitcoin-cli -regtest getblockchaininfo
bitcoin-cli -regtest getblockcount
bitcoin-cli -regtest getbestblockhash
```

## Terminal output

```
$ bitcoin-cli -regtest getblockchaininfo
{
  "chain": "regtest",
  "blocks": 0,
  "headers": 0,
  "bestblockhash": "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206a",
  "difficulty": 4.656542373906925e-10,
  "mediantime": 1296688602,
  "verificationprogress": 1,
  "initialblockdownload": false,
  "chainwork": "0000000000000000000000000000000000000000000000000000000000000000",
  "size_on_disk": 293,
  "pruned": false,
  "warnings": ""
}

$ bitcoin-cli -regtest getblockcount
0

$ bitcoin-cli -regtest getbestblockhash
0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206a
```

Rust `inspect_network` returned:

```
NetworkSnapshot {
  chain: "regtest",
  block_height: 0,
  best_block_hash: "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206a",
}
```

## Evidence references

- Polar screenshot: network "Week 1 Bitcoin Fundamentals" with one running Bitcoin Core node (status: Running).
- Screenshot of `bitcoin-cli -regtest getblockchaininfo` output showing `"chain": "regtest"`.
- `cargo test --test lab_01` — all 4 tests passed.

## Explanation

**Polar** is a desktop application for designing and launching local Bitcoin networks. It provides a visual interface to add nodes, configure connections, and start or stop the network without manually editing Docker Compose files.

**Docker** is the container runtime Polar uses under the hood. Each Bitcoin Core node runs inside an isolated container with its own filesystem, ports, and configuration. Docker keeps environments reproducible and separate from the host machine.

**Bitcoin Core** is the reference full-node implementation. In these labs it validates blocks, maintains the UTXO set, exposes JSON-RPC, and (with wallet enabled) signs transactions. Our Rust code drives it through `bitcoin-cli`.

**Regtest** (regression test network) is a private chain mode where the operator controls block production and coin issuance. Block rewards mature quickly for testing, addresses use the `bcrt1` prefix, and no real bitcoin is at risk. It is ideal for learning wallet, mempool, and confirmation behaviour without touching mainnet.
