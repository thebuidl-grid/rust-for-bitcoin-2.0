# Lab 09 — Multi-UTXO coin selection

## Commands used

TODO: Record funding, confirmation, spending, and decoding commands.
bitcoin@backend1:/$ bitcoin-cli -rpcwallet=miner sendtoaddress "$ALICE_ADDR" 0.4

## Terminal output

TODO: Show Alice's three UTXOs and the combined transaction inputs and outputs.
```bash
bitcoin@backend1:/$ bitcoin-cli -rpcwallet=miner sendtoaddress "$ALICE_ADDR" 0.4
f2d77b8b089a9efc0642a1ec03bc6e8fad071e276952b2c127c48055ab18e566
```

## Evidence references

TODO: Link screenshots or describe the attached evidence.
![img.png](evidence/img_9.png)

## Explanation

TODO: Explain input combination, change, fees, and the privacy implication.
Transactions combine multiple UTXOs as inputs to reach a payment total, returning surplus funds via a change output and leaving the difference as a miner fee.
This process has a privacy implication, as combining inputs on-chain reveals that the same wallet likely controlled those previously separate pieces of value