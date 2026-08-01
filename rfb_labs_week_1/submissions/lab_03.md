# Lab 03 — Coinbase maturity

## Commands used

`cargo test --test lab_03`

Live node commands:

`bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie generatetoaddress 1 <miner_address>`

`bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie getblockcount`

`bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=miner getbalances`

`bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=miner sendtoaddress <receiver_address> 1`

`bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie generatetoaddress 100 <miner_address>`

`bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie getblockcount`

`bitcoin-cli -regtest -rpccookiefile=/home/bitcoin/.bitcoin/regtest/.cookie -rpcwallet=miner getbalances`

## Terminal output

The public test suite passed, and the maturity evidence shows:

- height after first block: `1`
- balance after first block: `trusted: 0.0`, `untrusted_pending: 0.0`, `immature: 50.0`
- premature spend error: `Insufficient funds`
- final height: `101`
- final balance: `trusted: 50.0`, `untrusted_pending: 0.0`, `immature: 5000.0`

This shows the first coinbase reward became spendable at height 101, while later rewards are still immature.

## Evidence references

- `screenshots/lab3block.png` (mine 1 block and observe the first post-mine height)
- `screenshots/lab3-send-error.png` (premature 1 BTC spend attempt returns `Insufficient funds`)
- `screenshots/lab3-block-101.png` (after mining 100 more blocks)
- `screenshots/lab3-balances.png` (final balances showing trusted and immature values)

## Explanation

`COINBASE_MATURITY = 100` means a coinbase output must receive 100 more blocks before it can be spent. On a fresh chain, the first block is mined at height 1, so it becomes spendable only after the chain reaches height 101. That is why the lab mines 1 block, proves the reward is still immature, then mines 100 more blocks and shows the first reward is now part of the trusted balance.
