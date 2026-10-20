# Lab 08 — Block security

## Commands used

- Block header: `bitcoin-cli getblockheader "$BLOCK_HASH"`
- Initial confirmations: `bitcoin-cli -rpcwallet=receiver gettransaction "$TXID"`
- Additional mining: `bitcoin-cli generatetoaddress 5 "$MINER_ADDRESS"`
- Final confirmations: `bitcoin-cli -rpcwallet=receiver gettransaction "$TXID"`

## Terminal output

```text
confirming block:
  hash: 2c798aadd7f59683a44de5f886b71bbdcb31480641e3f36a4cc725838bed5110
  height: 102
  previousblockhash: 0e145831891a212f6f0e4ffc55054d3a88ad2f6c437941d001a3e3016fe9e6fb
  merkleroot: ddb5decd35898e7c77de6658765b366eb0097742ff7adbe8b5d22963c931f741
  nonce: 1
  difficulty: 4.6565423739069247e-10
  bits: 207fffff
  confirmations: 1
  chainwork: 00000000000000000000000000000000000000000000000000000000000000ce

payment confirmations before mining: 1
additional blocks mined: 5
payment confirmations after mining: 6
```

## Evidence references
![alt text](evidence/image-6.png)

## Explanation

The header's previous-block hash links this block to its parent, so altering an
earlier block would break every later link. The Merkle root commits the header to
the block's ordered transactions. The nonce is one field miners vary while searching
for a header hash below the target encoded by `bits`; accumulated chainwork measures
the proof of work in the chain. Five successor blocks raised the payment from one to
six confirmations. This increases the work required to reorganize it, but it cannot
turn an invalid transaction into a valid one because every node validates the
transaction independently.
