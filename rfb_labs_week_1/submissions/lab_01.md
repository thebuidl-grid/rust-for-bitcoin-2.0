# Lab 01 — Regtest network inspection

## Commands used

```bash
# Verify active chain, height, and tip via RPC helper and CLI
cargo test --test lab_01
bitcoin-cli -regtest getblockchaininfo
bitcoin-cli -regtest getblockcount
bitcoin-cli -regtest getbestblockhash
```

## Terminal output

```json
{
  "chain": "regtest",
  "blocks": 101,
  "bestblockhash": "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206"
}
```

```text
running 4 tests
test builds_verified_network_snapshot ... ok
test reads_best_block_hash ... ok
test reads_block_height ... ok
test reads_regtest_chain ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## Evidence references

- Executed `inspect_network` in `src/labs/lab01_network.rs` using `ProcessRpc`.
- Verified JSON-RPC response fields returned from `getblockchaininfo`, `getblockcount`, and `getbestblockhash`.
- Validated test suite execution in `tests/lab_01.rs`.

## Explanation

In Bitcoin software development and testing infrastructure, **Polar**, **Docker**, **Bitcoin Core**, and **regtest** represent distinct layers of the environment stack:

1. **Polar**: A graphical orchestration tool designed for Bitcoin and Lightning developers. It manages private regtest network topology, provisions nodes, configures RPC credentials, and routes Lightning channels visually.
2. **Docker**: The containerization runtime that isolates each Bitcoin Core instance (`bitcoind`) into an independent OS-level process namespace with dedicated ports, RPC credentials, and simulated P2P networks.
3. **Bitcoin Core**: The reference implementation of the Bitcoin protocol (`bitcoind` daemon and `bitcoin-cli` RPC client). It performs transaction validation, maintains the UTXO set database (LevelDB), executes consensus rules, and serves JSON-RPC endpoints.
4. **Regtest (Regression Test Mode)**: A private local blockchain network mode. Unlike mainnet or testnet/signet, regtest has a near-zero Proof-of-Work difficulty threshold (`bits: 207fffff`), allowing developers to mine blocks instantaneously on demand using `generatetoaddress` without consuming computational energy or requiring external peers.
