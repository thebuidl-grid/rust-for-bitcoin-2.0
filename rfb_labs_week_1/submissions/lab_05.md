# Lab 05 — Broadcast and mempool

## Commands used

cargo run --example lab05

Underlying bitcoin-cli RPCs invoked:
- sendtoaddress bcrt1q6nlqtswveesh573tml9mrwlczn66vfu0sjnaqn 1   (wallet: miner)
- getrawmempool
- gettransaction <txid>                                          (wallet: miner)
- getbalances                                                     (wallet: receiver)


## Terminal output

txid:                 a9e5849b95b19d9c08218953eeb0475c75b8b856f5838615bd37f37f6056647b
mempool contains tx:  true
sender confirmations: 0
sender amount:        -1
sender fee:           Some(-2.82e-5)
sender block_hash:    None
receiver balance: trusted=0 untrusted_pending=1 immature=0

## Evidence references

bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass getrawmempool
[
  "a9e5849b95b19d9c08218953eeb0475c75b8b856f5838615bd37f37f6056647b"
]

bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=receiver getbalances
{
  "mine": {
    "trusted": 0.00000000,
    "untrusted_pending": 1.00000000,
    "immature": 0.00000000
  },
  "lastprocessedblock": {
    "hash": "39192fb282dacb172ca1460f264fda7ed5070f7c14563f41f4d012fffe3457ec",
    "height": 102
  }
}
## Explanation

Signed is the earliest state, the tx has valid signatures satisfying its inputs' spending conditions but it only exists on the machine that created it, no node on the network has seen it, broadcast means that signed tx has been submitted to at least one node via RPC or relayed p2p, so it has now been announced to the network. mempool is the state of having been accepted by a node's validation checks and held in that node's memory pool of pending txs, waiting to be picked up by a miner. confirmed means the tx has been included in a mined block that is part of the best chain, and aditional mined blocks on top adds more confirmations, making reversal less likely.