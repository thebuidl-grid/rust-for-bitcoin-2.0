# Lab 04 — UTXOs and outpoints

## Commands used

cargo run --example lab04

Underlying bitcoin-cli RPCs invoked (wallet: miner):
- listunspent
- getbalances


## Terminal output

miner UTXO count: 1
selected UTXO:
  txid:            8efa1967e9a59084cfbba48f83d8d503c8c09c9797ef0c2d88d926f00c961f39
  vout:            0
  amount:          50
  confirmations:   101
  address:         Some("bcrt1qsfqwvhu2yn2ghu5yj2dsajdck38gykmk0nq7cn")
  script_pub_key:  00148240e65f8a24d48bf284929b0ec9b8b44e825b76
  spendable:       true
selected outpoint: 8efa1967e9a59084cfbba48f83d8d503c8c09c9797ef0c2d88d926f00c961f39:0
independent sum of spendable UTXOs: 50
wallet trusted balance (getbalances): 50
reconciled: true

## Evidence references

cargo run --example lab04

miner UTXO count: 1
selected UTXO:
  txid:            8efa1967e9a59084cfbba48f83d8d503c8c09c9797ef0c2d88d926f00c961f39
  vout:            0
  amount:          50
  confirmations:   101
  address:         Some("bcrt1qsfqwvhu2yn2ghu5yj2dsajdck38gykmk0nq7cn")
  script_pub_key:  00148240e65f8a24d48bf284929b0ec9b8b44e825b76
  spendable:       true
selected outpoint: 8efa1967e9a59084cfbba48f83d8d503c8c09c9797ef0c2d88d926f00c961f39:0
independent sum of spendable UTXOs: 50
wallet trusted balance (getbalances): 50
reconciled: true

bitcoin-cli -regtest -rpcport=18443 -rpcuser=polaruser -rpcpassword=polarpass -rpcwallet=miner listunspent

[
  {
    "txid": "8efa1967e9a59084cfbba48f83d8d503c8c09c9797ef0c2d88d926f00c961f39",
    "vout": 0,
    "address": "bcrt1qsfqwvhu2yn2ghu5yj2dsajdck38gykmk0nq7cn",
    "label": "mining",
    "scriptPubKey": "00148240e65f8a24d48bf284929b0ec9b8b44e825b76",
    "amount": 50.00000000,
    "confirmations": 101,
    "spendable": true,
    "solvable": true,
    "desc": "wpkh([3730b0af/84h/1h/0h/0/0]039283c2d8171581dd2631c7d592313de8d722e9a954926180cf6fe02716d5048b)#v7cwlkh6",
    "parent_descs": [
      "wpkh([3730b0af/84h/1h/0h]tpubDCecNNm1fFyNfQ8hcsCvAmKzmGrJraNhtiE7UoLYWWkYHSHnJ7J1NKMDR6repfPo7YtKU9drqN1h91wRcAdAJ8vwkxv6w9mZMx2N1fv8aJG/0/*)#dqhpzw4e"
    ],
    "safe": true
  }
]
## Explanation

A UTXO is an unspent transaction output, is bitcoin sitting in a transaction's output that has not been spent yet.
An outpoint is the unique identifier that points to one specific UTXO, it is the pair of txid and vout which is the transaction id it came from, plus the index of the output within that transaction