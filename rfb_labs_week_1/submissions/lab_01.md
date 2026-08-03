# Lab 01 — Regtest network inspection

## Commands used

```bash
cargo test --test lab_01
```

RPC methods called:
- `getblockchaininfo` - Retrieves blockchain information including chain name
- `getblockcount` - Gets the current block height
- `getbestblockhash` - Gets the hash of the best (tip) block

## Terminal output

```
running 4 tests
test builds_verified_network_snapshot ... ok
test reads_best_block_hash ... ok
![alt text](evidence/image-3.png)

test reads_block_height ... ok
![alt text](evidence/image-2.png)

test reads_regtest_chain ... ok
![alt text](evidence/image-1.png)

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

All tests pass successfully, confirming:
- Chain detection returns "regtest"
- Block height is correctly retrieved as u64
- Best block hash is retrieved as a string
- NetworkSnapshot correctly aggregates all three pieces of information

## Explanation

Lab 01 establishes the foundation for Bitcoin Core interaction through the RpcClient trait. The implementation demonstrates:

1. **Bitcoin Core RPC**: Bitcoin Core exposes a JSON-RPC interface via `bitcoin-cli`. Each function makes specific RPC calls to query network state.

2. **Regtest Network**: Regtest (regression test mode) is a private Bitcoin network useful for development and testing. It allows instant block creation without proof-of-work.

3. **Polar & Docker**: Polar manages Bitcoin Core nodes in Docker containers, providing isolated regtest networks for testing. Each node runs in its own container with configurable parameters.

4. **Network Inspection**: The `inspect_network` function demonstrates how to compose multiple RPC calls into a higher-level view of network state, returning validated data in a `NetworkSnapshot` struct.

The key insight is that all subsequent labs depend on this basic RPC communication pattern - making calls to Bitcoin Core and parsing responses into Rust data structures.
