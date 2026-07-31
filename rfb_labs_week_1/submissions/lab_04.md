# Lab 04 — UTXOs and outpoints

## Commands used

```bash
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=miner listunspent
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=miner listunspent | jq '[.[] | select(.spendable)] | map(.amount) | add'
docker exec --user bitcoin polar-n2-backend1 bitcoin-cli -regtest -rpcwallet=miner getbalance
```

## Terminal output

```text
{
  "txid": "352808782139e14c47df525c7186b2f6d4ee8632e69b6a2fd77fde63f02d0011",
  "vout": 0,
  "address": "bcrt1q60es4phrzmtqa5w6knzs8gqw97ykfn9kxfllwt",
  "scriptPubKey": "0014d3f30a86e316d60ed1dab4c503a00e2f8964ccb6",
  "amount": 50.00000000,
  "confirmations": 101,
  "spendable": true
}

Outpoint: 352808782139e14c47df525c7186b2f6d4ee8632e69b6a2fd77fde63f02d0011:0
Sum of spendable listunspent amounts: 50.00000000 BTC
getbalance:                            50.00000000 BTC
```

## Evidence references

Live wallet-scoped `listunspent` transcript from `miner`, plus an independent `jq` sum and
Bitcoin Core `getbalance` reconciliation.

## Explanation

A UTXO is a specific unspent transaction output containing an amount and locking script.
Its outpoint is the unique `txid:vout` coordinate shown above. Spending references that
coordinate and consumes the whole output; it does not subtract from an account row. The
wallet balance is a view derived by finding controlled, eligible UTXOs and summing their
values. Here there was exactly one spendable UTXO, so its 50 BTC value independently
matched the 50 BTC wallet balance; the later coinbase outputs were still immature.
