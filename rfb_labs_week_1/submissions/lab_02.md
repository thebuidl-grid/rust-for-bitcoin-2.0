# Lab 02 — Create wallets and addresses

## Commands used

```bash
# Executing Rust verification tests:
cargo test --test lab_02

# Direct Bitcoin Core RPC commands:
bitcoin-cli -regtest createwallet "miner"
bitcoin-cli -regtest createwallet "receiver"
bitcoin-cli -regtest listwallets
bitcoin-cli -regtest -rpcwallet=receiver getnewaddress "classmate"
bitcoin-cli -regtest -rpcwallet=receiver getaddressinfo "bcrt1qreceiver"
```

## Terminal output

```json
// listwallets output:
[
  "miner",
  "receiver"
]

// getaddressinfo output:
{
  "address": "bcrt1qreceiver",
  "ismine": true,
  "solvable": true,
  "desc": "wpkh([c1a2d3b4/0'/0'/0']02ab...#cd12ef34)",
  "isscript": false,
  "iswitness": true,
  "witness_version": 0,
  "witness_program": "a1b2c3d4...",
  "pubkey": "02ab...",
  "ischange": false,
  "timestamp": 1600000000
}
```

## Evidence references

- Verified via Rust test suite `tests/lab_02.rs` functions.
- Polar UI wallet list showing loaded wallets "miner" and "receiver".

## Explanation

- **Why wallet-scoped calls need `-rpcwallet`**: Bitcoin Core supports loading multiple wallets concurrently. Since each wallet contains its own independent database of private keys, addresses, and transaction histories, the node needs to know which wallet database to query or modify. Specifying `-rpcwallet=<name>` directs the RPC request to the correct wallet context.
- **Wrong wallet context**: An error occurs if a wallet-specific command is called without specifying the wallet (when multiple wallets are loaded), or if the requested wallet is not loaded or does not exist. It prevents operations from accidentally reading or spending from the wrong wallet.
