# Lab 03 — Coinbase maturity

## Commands used

TODO: Record mining, balance inspection, and premature-spend commands.
# 1. Mine initial block to the miner wallet address
bitcoin-cli -rpcwallet=miner generatetoaddress 1 "$(bitcoin-cli -rpcwallet=miner getnewaddress)"

# 2. Inspect wallet balances at height 1 (shows immature coinbase output)
bitcoin-cli -rpcwallet=miner getbalances

# 3. Attempt premature spend of immature coinbase funds (expected to fail)
bitcoin-cli -rpcwallet=miner sendtoaddress "$(bitcoin-cli -rpcwallet=receiver getnewaddress)" 1.0

# 4. Mine 100 additional blocks to satisfy the 100-block maturity rule
bitcoin-cli -rpcwallet=miner generatetoaddress 100 "$(bitcoin-cli -rpcwallet=miner getnewaddress)"

# 5. Inspect wallet balances at height 101 (confirming initial coinbase is now spendable)
bitcoin-cli -rpcwallet=miner getbalances

## Terminal output

TODO: Show balances at heights 1 and 101 plus the failed premature spend.
1. Balance Inspection at Height 1
{
  "mine": {
    "trusted": 0.00000000,
    "untrusted_pending": 0.00000000,
    "immature": 50.00000000
  }
}
2. Failed Premature Spend Output
error code: -6
error message:
Insufficient funds

3. Balance Inspection at Height 101
{
  "mine": {
    "trusted": 50.00000000,
    "untrusted_pending": 0.00000000,
    "immature": 5000.00000000
  }
}


## Evidence references

TODO: Link screenshots or describe the attached evidence.
1. crates/rfb-labs-week-1/tests/lab_03.rs: Integration test suite passing all 4 test scenarios (mines_requested_number_of_blocks, reads_nested_wallet_balances, preserves_insufficient_funds_error, and demonstrates_first_coinbase_becoming_spendable_at_height_101).

2. crates/rfb-labs-week-1/src/labs/lab03_maturity.rs: Rust implementation managing block generation via generatetoaddress, balance parsing via getbalances, error handling for premature spending, and verification of height progression (getblockcount).

## Explanation

TODO: Explain why the first coinbase reward becomes spendable at height 101.
1. Coinbase Maturity Rule: In Bitcoin consensus protocol rules, newly generated coins created in a coinbase transaction (the block reward and transaction fees) cannot be spent until they accumulate at least 100 confirmations on top of the block in which they were created.
2. Why at Height 101?
- At height 1, the block reward of 50 BTC is created. Its confirmation count is 1. The Bitcoin consensus rules categorize this output as immature, making its spendable (trusted) balance 0.0 BTC. 
- Any transaction attempting to spend an immature coinbase output will be rejected by node consensus rules with an Insufficient funds RPC error.
- Mining 100 additional blocks advances the chain tip to height 101. The original coinbase transaction at height 1 now has 101 confirmations ($101 - 1 + 1 = 101$), fulfilling the 100-confirmation threshold.
- Consequently, the 50 BTC reward transitions from immature to trusted spendable balance at height 101, while the subsequent 100 blocks (heights 2–101) remain immature (accumulating 5,000 immature BTC).
3. Consensus Security Purpose: The 100-block maturity constraint prevents funds derived from potential short-chain reorganizations from being spent prematurely. If a reorg orphans a block near the tip, any standard transaction referencing its coinbase output would instantly become invalid, triggering cascading invalidations across the mempool and network. The 100-block window ensures extreme chain depth before coinbase coins can enter circulation.