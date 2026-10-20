# Lab 03 — Coinbase maturity

## Commands used
- `bitcoin-cli -regtest -rpcwallet=miner getnewaddress "miner-address"` generates the mining address
- `bitcoin-cli -regtest generatetoaddress 1 <miner-address>` backs `mine_blocks` (mine the first block)
- `bitcoin-cli -regtest getblockcount` - reads height after mining
- `bitcoin-cli -regtest -rpcwallet=miner getbalances` backs `get_balances`
- `bitcoin-cli -regtest -rpcwallet=miner sendtoaddress <address> 1` backs `attempt_payment`
  (expected to fail while the only coin is immature)
- `bitcoin-cli -regtest generatetoaddress 100 <miner-address>` matures the first reward
- `bitcoin-cli -regtest getblockcount` / `getbalances` re-read height and balances after maturity

These compose into `demonstrate_coinbase_maturity`, which mines one block, records
height/balances, attempts (and expects to fail) a premature spend, then mines 100
more blocks and records the final height/balances.

## Terminal output

$ bitcoin-cli -regtest -rpcwallet=miner getnewaddress "miner-address"
bcrt1q7qcth6f0cq4w5tnfruqcxu20js6dcxrrmgum2z

$ bitcoin-cli -regtest generatetoaddress 1 bcrt1q7qcth6f0cq4w5tnfruqcxu20js6dcxrrmgum2z
[
"29c14725b9a01a3ae69db7f320650cee1d9258ce9638b73601aabfb32c337153"
]

$ bitcoin-cli -regtest getblockcount
588

$ bitcoin-cli -regtest -rpcwallet=miner getbalances
{
"mine": {
"trusted": 0.00000000,
"untrusted_pending": 0.00000000,
"immature": 6.25000000
}
}

$ bitcoin-cli -regtest -rpcwallet=miner sendtoaddress bcrt1q82tmmnmf77qymd5kg6k7ly3k4a45pfp8l9xxlr 1
error code: -6
error message:
Insufficient funds
$ bitcoin-cli -regtest generatetoaddress 100 bcrt1q7qcth6f0cq4w5tnfruqcxu20js6dcxrrmgum2z
[ ... 100 block hashes ... ]

$ bitcoin-cli -regtest getblockcount
688

$ bitcoin-cli -regtest -rpcwallet=miner getbalances
{
"mine": {
"trusted": 6.25000000,
"untrusted_pending": 0.00000000,
"immature": 346.87500000
}
}

## Evidence references
Captured directly from a local Bitcoin Core node running in regtest mode.
Note: this node's regtest chain had already passed several halvings before
this lab began (height 588, well past the first two 150-block halving
intervals on regtest), so the block subsidy here is 6.25 BTC rather than the
original 50 BTC used in the mocked unit tests, the maturity mechanics being
demonstrated (immature -> trusted after 100 blocks, and a rejected premature
spend in between) are identical regardless of the subsidy amount.

## Explanation (co-authored by Claude)

When a miner successfully mines a block, the reward for doing so, the coinbase: is paid out as a special first transaction in that block with no real inputs of its own. Bitcoin Core enforces a rule that this reward cannot be spent until it has at least 100 confirmations, meaning 100 additional blocks must be mined on top of the block containing it. This is why, in the evidence above, mining a single block at height 588 leaves the entire 6.25 BTC reward sitting in the immature balance rather than trusted, the wallet has the coins, but Bitcoin Core's consensus rules won't let them be spent yet.

The reason for this rule is protection against chain reorganizations. If a block were orphaned shortly after being mined, for example, because a competing chain with more accumulated proof of work reorganizes it away any transaction that had already spent that block's coinbase output would become invalid, since the coins it relied on would never have existed on the winning chain. By forcing miners to wait 100 blocks before spending a coinbase reward, the network ensures that reward is buried under enough additional proof of work to make such a reorg extremely unlikely, so any transactions built on top of it stay valid.

The evidence demonstrates: attempting sendtoaddress immediately after mining one block fails with an Insufficient funds error, even though getbalances clearly shows the wallet holds 6.25 BTC, because none of it is trusted. After mining 100 more blocks (bringing the chain from height 588 to 688), that same original reward crosses the 100-confirmation threshold and moves into the trusted balance, becoming spendable. Note that the immature figure at height 688 (346.875 BTC) reflects the newer coinbase rewards from those 100 freshly-mined blocks, which are themselves not yet mature only the original block's reward, now old enough, has become spendable.
