# Lab 01 — Regtest network inspection

## Commands used

### Rust Command
```bash
cargo run --example lab01_demo
```

### Bitcoin Core RPCs
```bash
bitcoin-cli -regtest getblockchaininfo  # Get chain name
bitcoin-cli -regtest getblockcount      # Get block height
bitcoin-cli -regtest getbestblockhash   # Get best block hash
```

## Terminal output

```
Chain:           regtest
Block Height:    1
Best Block Hash: 3c7e174e3a4eb984bfb936316c5dc0d7fdbd1dccec3d62548318fdb7ea44aea7
```

## Evidence references

Screenshots in `submissions/screenshots/`:
- `lab01_polar_network.png` - Polar network showing Bitcoin Core node running
- `lab01_rust_output.png` - Complete demo execution
- `lab01_node_details.png` - Node connection details

## Explanation

### Polar
Desktop app for managing Bitcoin/Lightning development networks. Provides visual network management and easy node access.

### Docker
Containerization platform that runs Bitcoin Core in isolated containers. Ensures consistent environments across systems.

### Bitcoin Core
The reference Bitcoin node software. Validates transactions, maintains the blockchain, and provides RPC APIs.

### Regtest (Regression Test Network)
Local testing network where:
- Blocks mine instantly on demand
- No proof-of-work difficulty
- Complete control over blockchain state
- Uses `bcrt1` address prefix
- Perfect for development without real funds or external dependencies
