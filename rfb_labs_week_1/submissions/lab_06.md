# Lab 06 — Transaction decoding

## Commands used

### Rust Command
```bash
cargo run --example lab06_demo
```

### Bitcoin Core Commands
```bash
bitcoin-cli getrawtransaction <txid> 2  # Decode with verbosity 2 (includes prevout)
bitcoin-cli -rpcwallet=miner sendtoaddress <address> <amount>  # Create transaction
bitcoin-cli generatetoaddress 1 <address>  # Mine to confirm
```

## Terminal output

### Transaction Details
```
TXID:   c6d76b6c2046316144ad93c2003fb32c2aad2f1cf00890beb60363e9cee4cb73
vSize:  141 bytes
```

### Input
```
Previous TXID: 29c57518ce43e83e0c41a4c3c5dfa9dd98c902237757fe52368fea12f2e20ea9:0
Value:         11 BTC
```

### Outputs
```
Payment Output:
  Value:   1.5 BTC
  Address: bcrt1qa2686ff6zq20evdqnfdwy8pjcjkmm47x2pdvq5

Change Output:
  Value:   9.4999718 BTC
  Address: bcrt1q30k289qstp8ltnuefd4lz5lac5gl7qdndqkspy
```

### Value Conservation
```
Total Input:   11 BTC
Total Output:  10.9999718 BTC
Miner Fee:     0.0000282 BTC

✓ Verified: 11 - 10.9999718 = 0.0000282
```

## Evidence references

Screenshots in `submissions/screenshots/`:
- `lab06_decoded_transaction.png` - Full transaction decode output
- `lab06_inputs_outputs.png` - Input and output details
- `lab06_fee_calculation.png` - Value conservation proof
- `lab06_rust_output.png` - Complete demo execution

## Explanation

### Value Conservation
Bitcoin enforces: **Total Inputs = Total Outputs + Fee**

In our transaction:
- Input: 11 BTC
- Outputs: 10.9999718 BTC (1.5 payment + 9.4999718 change)
- Fee: 0.0000282 BTC

The fee has no dedicated output. It's the implicit difference between inputs and outputs. Miners claim this fee when they include the transaction in a block.

### Why Verbosity 2?
Verbosity 2 includes `prevout` data showing input values. Without it, you can't calculate the fee because you only see which UTXOs are spent, not their values.
