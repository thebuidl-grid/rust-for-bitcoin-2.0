# Lab 05 — Broadcast and observe an unconfirmed payment

## Commands used

```bash
# Executing Rust verification tests:
cargo test --test lab_05

# Direct Bitcoin Core RPC commands:
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "bcrt1qreceiver" 1.0
bitcoin-cli -regtest getrawmempool
bitcoin-cli -regtest -rpcwallet=miner gettransaction "payment-txid"
bitcoin-cli -regtest -rpcwallet=receiver getbalances
```

## Terminal output

```json
// getrawmempool output:
[
  "payment-txid"
]

// gettransaction output (showing 0 confirmations):
{
  "txid": "payment-txid",
  "amount": -1.0,
  "fee": -0.00001,
  "confirmations": 0,
  "trusted": true,
  "walletconflicts": [],
  "time": 1600000000,
  "timereceived": 1600000000
}

// receiver's getbalances showing untrusted_pending:
{
  "mine": {
    "trusted": 0.0,
    "untrusted_pending": 1.0,
    "immature": 0.0
  }
}
```

## Evidence references

- Verified via Rust test suite `tests/lab_05.rs` functions.
- Checked transaction list and mempool size in Polar node UI.

## Explanation

- **Transaction States**:
  1. **Built and Signed**: The transaction is constructed by the wallet and signed with the private keys that unlock the input UTXOs. It remains local to the sender.
  2. **Broadcast**: The signed transaction is sent via network sockets to connected peers in the P2P network.
  3. **Mempool (Memory Pool)**: The node validates the broadcast transaction and stores it in memory (the mempool) while it waits for a miner to include it in a block.
  4. **Confirmed**: A miner successfully packages the transaction into a valid block and adds it to the blockchain.
- **Broadcast is not confirmation**: Broadcasting only advertises the transaction to the network. The transaction is not secured by proof-of-work until it is included in a block. Before confirmation, a transaction can be replaced (e.g. via RBF) or dropped from mempools, meaning it is not yet settled or guaranteed.
