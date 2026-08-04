# Lab 01 — Regtest network inspection

## Commands used

```bash
# 1. Start Bitcoin Core daemon in Regtest mode
bitcoind -regtest -rpcuser=polaruser -rpcpassword=polarpass -rpcport=18443 -fallbackfee=0.0001 -daemon

# 2. Query node chain info, height, and best block hash using bitcoin-cli
bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getblockchaininfo
bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getblockcount
bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getbestblockhash

# 3. Run verified Rust network inspection function
cargo test --test lab_01
```

## Terminal output

```text
$ bitcoin-cli -regtest -rpcuser=polaruser -rpcpassword=polarpass getblockchaininfo
{
  "chain": "regtest",
  "blocks": 0,
  "headers": 0,
  "bestblockhash": "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206",
  "difficulty": 4.656542373081379e-10,
  "verificationprogress": 1
}

$ cargo test --test lab_01
running 4 tests
test builds_verified_network_snapshot ... ok
test reads_best_block_hash ... ok
test reads_block_height ... ok
test reads_regtest_chain ... ok
test result: ok. 4 passed; 0 failed
```

## Evidence references

![Polar Regtest Node Inspection Screenshot](evidence/lab01_05.png)

## Explanation

**Polar, Docker, Bitcoin Core, and Regtest Roles:**
- **Polar**: A graphical orchestration tool designed for rapid local Bitcoin & Lightning network topology creation, offering visual node management and one-click RPC access.
- **Docker**: The container runtime engine that Polar uses to isolate and execute containerized Bitcoin Core daemon instances cleanly across host environments.
- **Bitcoin Core (`bitcoind`)**: The foundational reference software that implements the Bitcoin protocol rules, maintains the UTXO set, validates transactions/blocks, and exposes JSON-RPC.
- **Regtest (Regression Test Mode)**: A local, private Bitcoin network mode where block generation is instantaneous and on-demand (`generatetoaddress`), allowing developers to simulate transactions, mining, and network conditions deterministically without waiting for real proof of work.
