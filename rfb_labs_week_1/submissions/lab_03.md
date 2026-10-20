# Lab 03 — Coinbase maturity

## Commands used

```bash
MINER_ADDR=$(bitcoin-cli -regtest -rpcwallet=miner getnewaddress "mining")
RECEIVER_ADDR=$(bitcoin-cli -regtest -rpcwallet=receiver getnewaddress "classmate")

bitcoin-cli -regtest generatetoaddress 1 $MINER_ADDR
bitcoin-cli -regtest getblockcount
bitcoin-cli -regtest -rpcwallet=miner getbalances
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress $RECEIVER_ADDR 1
bitcoin-cli -regtest generatetoaddress 100 $MINER_ADDR
bitcoin-cli -regtest getblockcount
bitcoin-cli -regtest -rpcwallet=miner getbalances

cargo test --test lab_03
```

## Terminal output

After mining 1 block (height 1):

```
$ bitcoin-cli -regtest getblockcount
1

$ bitcoin-cli -regtest -rpcwallet=miner getbalances
{
  "mine": {
    "trusted": 0.00000000,
    "untrusted_pending": 0.00000000,
    "immature": 50.00000000
  }
}
```

Premature spend attempt:

```
$ bitcoin-cli -regtest -rpcwallet=miner sendtoaddress $RECEIVER_ADDR 1
error code: -4
error message: Insufficient funds
```

After mining 100 more blocks (height 101):

```
$ bitcoin-cli -regtest getblockcount
101

$ bitcoin-cli -regtest -rpcwallet=miner getbalances
{
  "mine": {
    "trusted": 50.00000000,
    "untrusted_pending": 0.00000000,
    "immature": 5000.00000000
  }
}
```

The first coinbase reward (50 BTC) moved from `immature` to `trusted`. Later rewards (100 blocks × 50 BTC = 5000 BTC) remain `immature`.

## Evidence references

- Screenshot of `getbalances` at height 1 showing 50 BTC immature and 0 trusted.
- Screenshot of the failed `sendtoaddress` error: "Insufficient funds".
- Screenshot of `getbalances` at height 101 showing 50 BTC trusted and 5000 BTC immature.
- `cargo test --test lab_03` — all 4 tests passed.

## Explanation

Bitcoin enforces `COINBASE_MATURITY = 100`, meaning a coinbase output cannot be spent until it is buried at least 100 blocks deep. This prevents spending rewards from blocks that might be reorganized away.

After mining one block, the 50 BTC reward exists but is immature — the wallet reports it under `immature`, not `trusted`. A spend attempt fails because spendable balance is zero.

Mining 100 additional blocks brings the first coinbase to depth 101 (1 confirming block + 100 more = 101 blocks total on chain, and the first coinbase is now 100 blocks deep). It becomes spendable (`trusted: 50`). Rewards from blocks 2–101 are still immature (each needs 100 confirmations). The lab conventionally mines 101 blocks total so the first reward matures while demonstrating that newer rewards remain locked.
