# Lab 03 — Coinbase maturity

## Commands used

```
cargo test --test lab_03

bitcoin-cli -regtest generatetoaddress 1 "<miner-address>"
bitcoin-cli -regtest getblockcount
bitcoin-cli -regtest -rpcwallet=miner getbalances
bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "<receiver-address>" 1
bitcoin-cli -regtest generatetoaddress 100 "<miner-address>"
bitcoin-cli -regtest getblockcount
bitcoin-cli -regtest -rpcwallet=miner getbalances
```

*RPCs are the ones issued by `mine_blocks`, `get_balances`, `attempt_payment`, and `demonstrate_coinbase_maturity` in `src/labs/lab03_maturity.rs`, verified against the mocked RPC client in `tests/lab_03.rs`. Run the `bitcoin-cli` lines against your live Polar regtest node to capture the terminal output below.*

## Terminal output

Captured against the live regtest node, continuing from Lab 02's state (miner address `bcrt1qtdwur5a220ta9f0lndtdqf45f0kmaplt3xea5l`, receiver address `bcrt1qx208aadpjxz7ftargmdy64amhslmycnjll2xxl`):

```
$ bitcoin-cli -regtest generatetoaddress 1 "bcrt1qtdwur5a220ta9f0lndtdqf45f0kmaplt3xea5l"
[ "3dd882d2ca7a04c31a85a227a68f378c4371e9d5f47ea6ac85981b9ba20463e0" ]

$ bitcoin-cli -regtest getblockcount
1

$ bitcoin-cli -regtest -rpcwallet=miner getbalances
{
  "mine": {
    "trusted": 0.00000000,
    "untrusted_pending": 0.00000000,
    "immature": 50.00000000
  },
  ...
}

$ bitcoin-cli -regtest -rpcwallet=miner sendtoaddress "bcrt1qx208aadpjxz7ftargmdy64amhslmycnjll2xxl" 1
error code: -6
error message:
Insufficient funds

$ bitcoin-cli -regtest generatetoaddress 100 "bcrt1qtdwur5a220ta9f0lndtdqf45f0kmaplt3xea5l"
[
  "4d35bbf2bd0dfd2ea700220a99c63166f9e485e217a81a553a0ecf31aae88f6d",
  "074b4f0a968d5e88c80836a038cf536dc9e221560a12b40296f64a0af7facd84",
  "5fd636bf446fbe80fd709cd93f5093512ce128a8234de8368fa4af5c60c3cb61",
  ... (94 more block hashes omitted) ...
  "733fad1a0c15fcd1ba27eb234f54c34cb145da17562bcae4a32f776a76b43c5c",
  "70d7fa6636c7da005fdc4202dd7a00a525546cbe62f816940ef4a93875ab48e1",
  "20774b91b25a63e16a078d32fb2306c9461ff0bd51e22f673c3b9c4d96db5f7d"
]

$ bitcoin-cli -regtest getblockcount
101

$ bitcoin-cli -regtest -rpcwallet=miner getbalances
{
  "mine": {
    "trusted": 50.00000000,
    "untrusted_pending": 0.00000000,
    "immature": 5000.00000000
  },
  ...
}
```

At height 1: the block-1 coinbase (50 BTC) is entirely `immature`; a 1 BTC send fails with RPC error `-6 Insufficient funds` because no trusted/spendable balance exists yet. After mining 100 more blocks (height 101): that same 50 BTC has moved from `immature` to `trusted`, and 100 further immature coinbases (5000 BTC total) sit behind it, each still needing its own 100 confirmations.

## Evidence references

Evidence is the live terminal output above, captured directly via `docker exec bitcoind-lab-a bitcoin-cli ...` against a real regtest node (not a screenshot — this session ran headlessly, no Polar GUI was open).

## Explanation

Every block's reward (the "coinbase" transaction, containing the newly-minted BTC plus any fees) is created *inside that block itself* — but Bitcoin Core refuses to let a wallet spend a coinbase output until it has **100 confirmations**. This rule is enforced by the consensus protocol, not just a wallet preference: a transaction spending an immature coinbase is invalid and every node will reject it.

The reason for the rule is chain-reorganization safety. If a shorter, competing chain could later overtake the one you mined on (see Lab 10), the block containing your coinbase reward could become orphaned/stale, and the "coins" it created would simply cease to exist. Deep reorgs (100+ blocks) are considered astronomically unlikely, so 100 confirmations is treated as effectively permanent. Requiring 100 confirmations before that reward is spendable prevents a wallet from spending money that a reorg could later erase.

Concretely: block 1's coinbase reaches 100 confirmations once block 101 is mined (block 1 itself is confirmation #1, and each block mined on top adds one more — block 101 is the 100th confirmation). That's exactly what the terminal output shows: at height 1, that 50 BTC sits in `immature` and a spend attempt fails with `Insufficient funds`; at height 101, the same 50 BTC has moved into `trusted` and is spendable. Every later block's own coinbase follows the identical rule, maturing 100 blocks after *it* was mined — which is why, at height 101, the other 100 coinbases mined along the way are still `immature` (5000 BTC's worth, none of them 100 confirmations deep yet).
