# Lab 07 — Confirmation and block membership

## Commands used

```bash
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest generatetoaddress 1 bcrt1q60es4phrzmtqa5w6knzs8gqw97ykfn9kxfllwt
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest getrawmempool
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=receiver getbalances
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=receiver gettransaction 7d962b2ffea59f7651f809900184954e2b9dd6c32f41f2c70594df0f1bdb8152
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest getblock 2ed108f1c1b69d4f406f6c2354cafa20b63aafe54d9d348704b7bfdbca9c12ae 1
```

## Terminal output

```text
Generated block: 2ed108f1c1b69d4f406f6c2354cafa20b63aafe54d9d348704b7bfdbca9c12ae
getrawmempool: []
receiver balance: trusted 1.00000000, untrusted_pending 0.00000000

gettransaction:
  txid: 7d962b2ffea59f7651f809900184954e2b9dd6c32f41f2c70594df0f1bdb8152
  confirmations: 1
  blockhash: 2ed108f1c1b69d4f406f6c2354cafa20b63aafe54d9d348704b7bfdbca9c12ae

getblock height: 102
tx: [
  "59335bc511966c43f3a8413270c2440100475c0c5a971b94ac97605ecbf9034b",
  "7d962b2ffea59f7651f809900184954e2b9dd6c32f41f2c70594df0f1bdb8152"
]
```

## Evidence references

Sequential live transcript: mining occurred once, then the empty mempool, receiver wallet,
transaction record, and verbose block were queried.

## Explanation

Mining did not change the transaction or its TXID. It moved from the mempool into block
`2ed108...12ae`, which became the active tip. The mempool was then empty, the receiver's
1 BTC moved from pending to trusted, and `gettransaction` reported one confirmation. The
same TXID in the block's transaction list confirmed that this was the containing block.
