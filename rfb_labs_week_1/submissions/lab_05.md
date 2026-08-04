# Lab 05 — Broadcast and mempool

## Commands used

### Rust Command
```bash
cargo run --example lab05_demo
```

### Bitcoin Core Commands
```bash
bitcoin-cli -rpcwallet=miner sendtoaddress <address> <amount>  # Send payment
bitcoin-cli getrawmempool                                      # Check mempool
bitcoin-cli -rpcwallet=miner gettransaction <txid>             # Check tx status
bitcoin-cli -rpcwallet=receiver getbalances                    # Check receiver balance
```

## Terminal output

### Unconfirmed Transaction
```
TXID:                 a3407a7fcce79fd267a01181c573976eb9106d8f797ac64ab38ce903b5e4d6a0
In Mempool:           true
```

### Sender View
```
Confirmations:        0
Amount:               -1 BTC
Fee:                  -0.0000282 BTC
Block Hash:           (none - unconfirmed)
```

### Receiver Balance
```
Trusted:              0 BTC
Untrusted Pending:    1 BTC
Immature:             0 BTC
```

## Evidence references

Screenshots in `submissions/lab05_screenshots/`:
- `get_mempool.png` - Transaction in mempool
- `lab05_receiver_balance.png` - Receiver showing untrusted_pending balance


## Explanation

### Transaction States
1. **Signed**: Transaction created and signed, but not broadcast
2. **Broadcast**: Transaction sent to network and enters mempool
3. **Mempool**: Transaction waiting to be included in a block (0 confirmations)
4. **Confirmed**: Transaction included in a mined block (≥1 confirmation)

### The Mempool
The mempool holds unconfirmed transactions. When you broadcast a transaction, it enters the mempool where miners can see it and include it in the next block. Each node maintains its own mempool.

### Untrusted Pending Balance
When a receiver gets an unconfirmed transaction:
- Shows in `untrusted_pending` balance (not `trusted`)
- Can't be spent until confirmed
- Could be reversed (double-spent) before confirmation
- Moves to `trusted` after mining into a block
