# Lab 08 — Block security

## Commands used

```bash
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest getblockheader 2ed108f1c1b69d4f406f6c2354cafa20b63aafe54d9d348704b7bfdbca9c12ae
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=receiver gettransaction 7d962b2ffea59f7651f809900184954e2b9dd6c32f41f2c70594df0f1bdb8152
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest generatetoaddress 5 bcrt1q60es4phrzmtqa5w6knzs8gqw97ykfn9kxfllwt
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=receiver gettransaction 7d962b2ffea59f7651f809900184954e2b9dd6c32f41f2c70594df0f1bdb8152
```

## Terminal output

```text
hash:              2ed108f1c1b69d4f406f6c2354cafa20b63aafe54d9d348704b7bfdbca9c12ae
height:            102
previousblockhash: 51857abd3c8bdca1afb3238fc6454c0cbf258d0790d72e7ad03003279b6e2f93
merkleroot:        88a89534e9d793eb0deaa66134a9f96548e0fb054ef905c22e4c92cc7d323dd0
nonce:             0
bits:              207fffff
difficulty:        4.656542373906925E-10
confirmations:     1
chainwork:         00000000000000000000000000000000000000000000000000000000000000ce

Five generated blocks: 73731f...d601, 607af2...74a2, 1a3d29...4ba1,
                       24be84...c414, 465e2f...7876
Payment confirmations after mining: 6
```

## Evidence references

Live header and receiver-wallet transcript from the same confirming block and TXID used in
Lab 07, before and after exactly five additional blocks.

## Explanation

`previousblockhash` links height 102 to its parent, so changing prior history would also
change every descendant header. The Merkle root commits the block header to its full
transaction set, including the payment. Miners vary header fields such as the nonce while
searching for a hash below the target encoded by `bits`; regtest's target is intentionally
easy, so nonce 0 can be valid. `chainwork` measures accumulated proof of work, which nodes
use to compare valid branches. Five descendants increased the payment from one to six
confirmations, increasing the work an alternative branch must surpass. Confirmations
cannot make an invalid transaction valid—every node still validates scripts and consensus
rules before accepting either the transaction or its block.
