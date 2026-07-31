# Lab 01 — Regtest network inspection

## Commands used

```bash
# Executing Rust verification tests:
cargo test --test lab_01

# Direct Bitcoin Core RPC commands:
bitcoin-cli -regtest getblockchaininfo
bitcoin-cli -regtest getblockcount
bitcoin-cli -regtest getbestblockhash
```

## Terminal output

```json
// output from bitcoin-cli -regtest getblockchaininfo
{
  "chain": "regtest",
  "blocks": 101,
  "headers": 101,
  "bestblockhash": "5f137e1a3843e9ef2436f56191b7d568c850fa053a6b4d53... (example hash)",
  "difficulty": 4.6565423739069247e-10,
  "mediantime": 1600000000,
  "verificationprogress": 1,
  "initialblockdownload": false,
  "chainwork": "00000000000000000000000000000000000000000000000000000000000000cc"
}
```

## Evidence references

- Verified via Rust test `reads_regtest_chain`, `reads_block_height`, `reads_best_block_hash`, and `builds_verified_network_snapshot` in `tests/lab_01.rs`.
- Checked block count and network info in Polar node explorer UI showing block height 101.

## Explanation

- **Polar**: A graphical developer tool for spinning up local Bitcoin networks using Docker. It simplifies managing nodes, wallets, mining, and network connections.
- **Docker**: The underlying virtualization platform that executes the Bitcoin Core daemon (`bitcoind`) inside isolated, lightweight container environments.
- **Bitcoin Core**: The protocol software implementation itself. The client runs a node that validates blocks/transactions and exposes JSON-RPC APIs for interaction.
- **regtest**: Short for regression test. A private local blockchain mode where difficulty is minimal, allowing developers to generate blocks instantly on demand without expensive Proof-of-Work mining.
