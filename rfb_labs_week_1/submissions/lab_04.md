# Lab 04 — UTXOs and outpoints

## Commands used

### Rust Command
```bash
cargo run --example lab04_demo
```

### Bitcoin Core Commands
```bash
# List all UTXOs
bitcoin-cli -rpcwallet=miner listunspent

# Get wallet balances
bitcoin-cli -rpcwallet=miner getbalances

# Get transaction details
bitcoin-cli getrawtransaction <txid> true
```

## Terminal output

### Selected Spendable UTXO
```
txid:           42d2b590ba684e60fcb4be53dab1b8f493769dff7cba19839150172705eb484e
vout:           0
amount:         50 BTC
confirmations:  101
address:        bcrt1qurtksfs0hx3kzvkgh37kv97r025hc3eusvndqq
scriptPubKey:   0014e0d768260fb9a36132c8bc7d6617c37aa97c473c
spendable:      true
```

### Outpoint
```
42d2b590ba684e60fcb4be53dab1b8f493769dff7cba19839150172705eb484e:0
```

### Balance Reconciliation
```
Our UTXO sum:       50 BTC
Core trusted:       50 BTC
✓ MATCH
```

## Evidence references

Screenshots in `submissions/screenshots/`:
- `utxo_from_miner.png` - All UTXOs from miner wallet
- `selected_utxo.png` - Details of selected spendable UTXO


## Explanation

### What is a UTXO?
A UTXO (Unspent Transaction Output) is an output from a past transaction that hasn't been spent yet. It's a chunk of Bitcoin you can use as input in a new transaction.

### What is an Outpoint?
An outpoint is a unique identifier for a UTXO, consisting of:
- **txid**: The transaction ID that created this output
- **vout**: The output index within that transaction

Format: `txid:vout` (e.g., `42d2b59...:0`)

### Why is a wallet balance the sum of UTXOs?
Bitcoin doesn't use account balances like traditional banking. Instead:

1. **No accounts**: There's no "balance" field stored anywhere for your wallet
2. **Collection of UTXOs**: Your wallet controls multiple UTXOs scattered across the blockchain
3. **Sum calculation**: Your balance is calculated by summing all spendable UTXOs you control
4. **Spending UTXOs**: When you spend, you consume entire UTXOs as transaction inputs and create new UTXOs as outputs

This is fundamentally different from an account-based system where a balance is stored and updated. In Bitcoin, the blockchain is a collection of transactions with outputs, and your balance is derived by identifying which unspent outputs belong to your wallet's keys.
