# Lab 03 — Coinbase maturity

## Commands used

<!-- TODO: Record mining, balance inspection, and premature-spend commands. -->
```bash
bitcoin-cli -rpcwallet=miner generatetoaddress <count> <address>  # Mine blocks
bitcoin-cli -rpcwallet=miner getbalances                          # Check balances
bitcoin-cli -rpcwallet=miner sendtoaddress <address> <amount>    # Send Funds 
```

## Terminal output

<!-- TODO: Show balances at heights 1 and 101 plus the failed premature spend. -->
summary:
After mining block 1 the height increased to 2 and the immature balance was 50BTC, then i try to send 10BTC to the reciver address and it failed, then after generating additional 100 blocks , the immature balance becomes 5000BTC and the trusted 50BTC, meaning the the first reward is mature for spending, after sending 10BTC to reciever address, the balnce of miner remains 39.99BTC bcos of fee and receiver wallet untrusted_pending increased to 10BTC.

BALANCE AT HEIGHT 1 and 2

```bash
bitcoin@backend1:/$ bitcoin-cli -rpcwallet=miner getbalances
{
  "mine": {
    "trusted": 0.00000000,
    "untrusted_pending": 0.00000000,
    "immature": 0.00000000
  },
  "lastprocessedblock": {
    "hash": "675e7afb6d02b22888990c97bb6b7ca49b248d28abdb3b02aaaf420b2d5366e1",
    "height": 1
  }
}
bitcoin@backend1:/$ bitcoin-cli -rpcwallet=miner generatetoaddress 1 bcrt1qn893ldl3w0zt5myjm0lxh3kpreedtwtnsc0272
[
  "0f4b53b5c7f59891f01b8bdb792c1ec4329337fe7f33ffbc920526e702bdb820"
]
bitcoin@backend1:/$ bitcoin-cli -rpcwallet=miner getbalances
{
  "mine": {
    "trusted": 0.00000000,
    "untrusted_pending": 0.00000000,
    "immature": 50.00000000
  },
  "lastprocessedblock": {
    "hash": "0f4b53b5c7f59891f01b8bdb792c1ec4329337fe7f33ffbc920526e702bdb820",
    "height": 2
  }
}
```


TRYING TO SEND IMMATURE FUND
```bash
bitcoin@backend1:/$ bitcoin-cli -rpcwallet=miner sendtoaddress bcrt1qcs67lj6q6up4nadjggckcpek27wtrkgz8h58wr 10
error code: -6
error message:
Insufficient funds
```

BALANCE AT HEIGHT 101
```bash
bitcoin@backend1:/$ bitcoin-cli -rpcwallet=miner getbalances
{
  "mine": {
    "trusted": 50.00000000,
    "untrusted_pending": 0.00000000,
    "immature": 5000.00000000
  },
  "lastprocessedblock": {
    "hash": "30aee7e2f5b8671723de4af2eaf8776736522ad961aef220e10d79b0de3541a7",
    "height": 102
  }
}
bitcoin@backend1:/$ bitcoin-cli -rpcwallet=miner sendtoaddress bcrt1qcs67lj6q6up4nadjggckcpek27wtrkgz8h58wr 10
5973f6d0548485a3abf7e0afb640a3a52e1ed515b2eb94a32e9047174368d53f
bitcoin@backend1:/$ bitcoin-cli -rpcwallet=miner getbalances
{
  "mine": {
    "trusted": 39.99997180,
    "untrusted_pending": 0.00000000,
    "immature": 5000.00000000
  },
  "lastprocessedblock": {
    "hash": "30aee7e2f5b8671723de4af2eaf8776736522ad961aef220e10d79b0de3541a7",
    "height": 102
  }
}
bitcoin@backend1:/$ bitcoin-cli -rpcwallet=receiver getbalances
{
  "mine": {
    "trusted": 0.00000000,
    "untrusted_pending": 10.00000000,
    "immature": 0.00000000
  },
  "lastprocessedblock": {
    "hash": "30aee7e2f5b8671723de4af2eaf8776736522ad961aef220e10d79b0de3541a7",
    "height": 102
  }
}
bitcoin@backend1:/$ 
```

## Evidence references

<!-- TODO: Link screenshots or describe the attached evidence. -->
Screenshot 1 shows miner balance before anf after mining first block

Screenshot 2 shows error when trying to spend immature BTC


Screenshot 3 shows miner mining 100 more blocks so the first block mine can reach maturity

Screenshot 4 shows balance of miner after extra 100 block so first rewards get matured, and balance after sending 10BTC to reciever, and balance of receiver after receiving the 10BTC.

Screenshot 5 shows the test result lab03 implementation


## Explanation

<!-- TODO: Explain why the first coinbase reward becomes spendable at height 102. -->

Coinbase maturity rule: Bitcoin requires a coinbase reward to have 100 confirmations before it can be spent. This prevents issues if a chain reorg were to invalidate that block later.

Why height 102:

Block 2 creates the coinbase reward (1st confirmation).
Blocks 3 through 102 each add one more confirmation on top of it.
At block 102, block 1's coinbase reward finally has 100 confirmations → it becomes spendable.

So: reward is born at height 2, matures 100 blocks later, spendable starting at height 102.
