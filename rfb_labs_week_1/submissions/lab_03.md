# Lab 03 — Coinbase maturity

## Commands used

```bash
cargo test --test lab_03
```

RPC methods called:
- `generatetoaddress <count> <address>` - Mine blocks to a specific address
- `getbalances` - Inspect trusted, untrusted_pending, and immature balances
- `sendtoaddress <address> <amount>` - Attempt a payment
- `getblockcount` - Get current block height

## Terminal output

```
running 4 tests
test demonstrates_first_coinbase_becoming_spendable_at_height_101 ... ok
test mines_requested_number_of_blocks ... ok
test preserves_insufficient_funds_error ... ok
test reads_nested_wallet_balances ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Evidence references

All tests pass successfully, demonstrating:
- Mining 1 block at height 0 results in immature coinbase rewards
- Attempting to spend immature coins fails with "Insufficient funds" error
- Mining 100 more blocks (total 101) makes the first block's rewards spendable
- Balances correctly reflect immature vs trusted funds

## Explanation

Lab 03 demonstrates Bitcoin's coinbase maturity rule - a critical consensus rule:

1. **Coinbase Rewards**: Mining rewards start immature and require 100 more blocks to mature. This prevents blockchain reorganizations from allowing double-spending of block rewards.

2. **Balance Categories**: Bitcoin Core reports three balance types:
   - **Trusted**: Fully mature, spendable funds
   - **Untrusted pending**: Received but with 0 confirmations
   - **Immature**: Recently mined (< 101 confirmations)

3. **The 100-Block Rule**: At block height 1, block 0's reward is immature. At block height 101, it has 101 confirmations and becomes spendable. This 100-block gap prevents chain reorganization attacks.

4. **Practical Impact**: New miners cannot immediately spend their rewards - they must wait for network consensus to solidify their blocks. This is a core security mechanism in Bitcoin.
