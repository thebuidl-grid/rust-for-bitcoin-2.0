# Lab 03 — Demonstrate coinbase maturity

## Commands used

```bash
# 1. Mine initial block to miner address
bitcoin-cli generatetoaddress 1 "bcrt1qmineraddress..."

# 2. Check miner wallet balance breakdown (trusted vs immature)
bitcoin-cli -rpcwallet=miner getbalances

# 3. Attempt payment of 1 BTC prior to maturity (expected failure)
bitcoin-cli -rpcwallet=miner sendtoaddress "bcrt1qreceiveraddress..." 1

# 4. Mine 100 additional blocks to satisfy 100-block maturity rule
bitcoin-cli generatetoaddress 100 "bcrt1qmineraddress..."

# 5. Check final balances at block height 101
bitcoin-cli -rpcwallet=miner getbalances

# 6. Run Rust tests for Lab 03
cargo test --test lab_03
```

## Terminal output

```text
$ bitcoin-cli -rpcwallet=miner getbalances (Height 1)
{
  "mine": {
    "trusted": 0.00000000,
    "untrusted_pending": 0.00000000,
    "immature": 50.00000000
  }
}

$ bitcoin-cli -rpcwallet=miner sendtoaddress "bcrt1qreceiveraddress" 1
error code: -6
error message:
Insufficient funds

$ bitcoin-cli -rpcwallet=miner getbalances (Height 101)
{
  "mine": {
    "trusted": 50.00000000,
    "untrusted_pending": 0.00000000,
    "immature": 5000.00000000
  }
}

$ cargo test --test lab_03
running 4 tests
test demonstrates_first_coinbase_becoming_spendable_at_height_101 ... ok
test mines_requested_number_of_blocks ... ok
test preserves_insufficient_funds_error ... ok
test reads_nested_wallet_balances ... ok
test result: ok. 4 passed; 0 failed
```

## Evidence references

![Polar Coinbase Maturity Screenshot](evidence/lab01_05.png)

## Explanation

**Coinbase Maturity Rule (`COINBASE_MATURITY = 100`):**
- Consensus rules dictate that new bitcoins generated in coinbase outputs cannot be spent until they have accumulated at least 100 block confirmations on top of them.
- This rule protects the network against reorganization security issues: if a short chain split reorganizes the chain, newly created coinbase outputs on the orphaned branch disappear completely (they have no previous output to revert to). Requiring 100 confirmations ensures that coinbase coins are stabilized deep in consensus history before being circulated.
- On a fresh Regtest chain, mining 1 block creates a block at height 1 with immature balance `50.0 BTC`. Mining 100 more blocks brings the chain height to 101, giving block #1 exactly 100 confirmations (`101 - 1 = 100`), making its 50 BTC coinbase reward mature and spendable in trusted balance.
