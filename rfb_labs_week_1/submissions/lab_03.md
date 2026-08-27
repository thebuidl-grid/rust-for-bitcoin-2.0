# Lab 03 — Coinbase maturity

## Commands used

cargo run --example lab03 -- bcrt1qsfqwvhu2yn2ghu5yj2dsajdck38gykmk0nq7cn bcrt1q6nlqtswveesh573tml9mrwlczn66vfu0sjnaqn

Underlying bitcoin-cli RPCs invoked (wallet: miner unless noted):
- generatetoaddress 1 bcrt1qsfqwvhu2yn2ghu5yj2dsajdck38gykmk0nq7cn
- getblockcount
- getbalances
- sendtoaddress bcrt1q6nlqtswveesh573tml9mrwlczn66vfu0sjnaqn 1   (failed: -6 Insufficient funds)
- generatetoaddress 100 bcrt1qsfqwvhu2yn2ghu5yj2dsajdck38gykmk0nq7cn
- getblockcount
- getbalances

## Terminal output

height after first block: 2
balance after first block: trusted=0 untrusted_pending=0 immature=50
premature 1 BTC spend error: error code: -6
error message:
Insufficient funds
final height: 102
final balance: trusted=50 untrusted_pending=0 immature=5000

## Evidence references

bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass getblockcount
102

bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=miner getbalances
{
  "mine": {
    "trusted": 50.00000000,
    "untrusted_pending": 0.00000000,
    "immature": 5000.00000000
  },
  "lastprocessedblock": {
    "hash": "39192fb282dacb172ca1460f264fda7ed5070f7c14563f41f4d012fffe3457ec",
    "height": 102
  }
}
## Explanation

This Polar node had already mined 1 block at creation (height 1), so after mining 1 + 100 = 101 additional blocks in this lab, the final height is 102, not 101. The maturity behavior is identical regardless of starting height: COINBASE_MATURITY = 100 means a coinbase output needs 100 confirmations (101 total blocks including itself) before it's spendable — my first-block reward went from immature (0 confirmations) to spendable exactly once 100 more blocks were mined on top of it. 