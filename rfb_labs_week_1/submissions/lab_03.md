# Lab 03 — Coinbase maturity

## Commands used

### Rust Command
```bash
cargo run --example lab03_demo
```

### Bitcoin Core Commands
```bash
bitcoin-cli -rpcwallet=miner generatetoaddress <count> <address>  # Mine blocks
bitcoin-cli -rpcwallet=miner getbalances                          # Check balances
bitcoin-cli -rpcwallet=miner sendtoaddress <address> <amount>     # Attempt payment
bitcoin-cli getblockcount                                         # Check height
```

## Terminal output

```
After mining block 1:
  Block Height:        2
  Trusted Balance:     0 BTC
  Immature Balance:    50 BTC

Attempted to spend 1 BTC:
  ✗ Error: Insufficient funds (or invalid address)

After mining 100 more blocks:
  Block Height:        102
  Trusted Balance:     50 BTC
  Immature Balance:    5000 BTC
```

**Result**: First coinbase reward (block 2) became spendable at block 102 after 100 confirmations.

## Evidence references

Screenshots in `submissions/lab03_screenshots/`:
- `get_balances.png` - Block 2, 
- `against_polar_output.png` 

## Explanation

### COINBASE_MATURITY = 100
Coinbase rewards cannot be spent until 100 additional blocks are mined on top. This prevents spending from orphaned blocks during chain reorganizations.

### Why mine 101 blocks on a fresh chain?
- **Block 1**: Contains 50 BTC reward (immature, 0 confirmations)
- **Blocks 2-100**: Add confirmations (1-99 confirmations)
- **Block 101**: Reaches 100 confirmations → reward becomes spendable

At height 101, the block 1 reward has exactly 100 confirmations (blocks 2-101).

### Our Lab (started at height 1)
- **Block 2**: Our first reward (immature)
- **Blocks 3-101**: Add confirmations
- **Block 102**: Our reward matures (100 confirmations)

### Balance Types
- **Trusted**: Mature, spendable coins
- **Untrusted Pending**: Unconfirmed transactions (0 confirmations)
- **Immature**: Coinbase rewards with <100 confirmations

As new blocks are mined, coinbase rewards move from immature → trusted after 100 confirmations.
