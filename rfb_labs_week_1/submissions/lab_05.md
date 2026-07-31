# Lab 05 — Broadcast and mempool

## Commands used

```bash
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=miner sendtoaddress bcrt1qduu2p93zkvp0v69uqglj0dlaam0rwplhk6gf3u 1
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest getrawmempool
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=miner gettransaction 7d962b2ffea59f7651f809900184954e2b9dd6c32f41f2c70594df0f1bdb8152
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=receiver getbalances
```

## Terminal output

```text
TXID: 7d962b2ffea59f7651f809900184954e2b9dd6c32f41f2c70594df0f1bdb8152
getrawmempool: ["7d962b2ffea59f7651f809900184954e2b9dd6c32f41f2c70594df0f1bdb8152"]

sender gettransaction:
{
  "amount": -1.00000000,
  "fee": -0.00002820,
  "confirmations": 0,
  "blockhash": null
}

receiver getbalances:
mine: { trusted: 0.00000000, untrusted_pending: 1.00000000, immature: 0.00000000 }
```

## Evidence references

Live transcript captured before mining any confirmation block. The same TXID appears in
both the send result and the node's local mempool.

## Explanation

The miner wallet first selected inputs, built the outputs, and signed the transaction.
Broadcast then delivered those bytes to the node. The node validated the transaction for
policy/consensus and admitted it to its local mempool, but that pool is not the blockchain.
The observed zero confirmations, null block hash, and receiver `untrusted_pending: 1`
prove it was merely broadcast and locally accepted. Confirmation requires a miner to
include that same serialized transaction in a valid block on the active chain.
